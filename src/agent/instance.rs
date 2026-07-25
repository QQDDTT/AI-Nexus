use crate::core::interfaces::{MemoryStore, Persona, SkillRegistry};
use std::sync::Arc;
use crate::gemini::types::{Content, GenerateRequest, Part, Tool};

pub struct AgentInstance {
    pub agent_id: String,
    pub owner_id: String,
    pub capability_requirement: String,
    pub persona: Persona,
    pub memory: Box<dyn MemoryStore>,
    pub skill_registry: Arc<dyn SkillRegistry>,
}

impl AgentInstance {
    pub fn new(
        agent_id: String, 
        owner_id: String, 
        capability_requirement: String,
        persona: Persona, 
        memory: Box<dyn MemoryStore>, 
        skill_registry: Arc<dyn SkillRegistry>
    ) -> Self {
        Self {
            agent_id,
            owner_id,
            capability_requirement,
            persona,
            memory,
            skill_registry,
        }
    }

    /// 根据 Agent 的能力标签要求与 Model Router 配置解析具体的主/备用模型名称
    pub fn resolve_target_model(&self, routing_map: Option<&serde_json::Value>) -> String {
        let req = if self.capability_requirement.is_empty() {
            "Tier-1-Logic"
        } else {
            &self.capability_requirement
        };

        if let Some(map) = routing_map {
            if let Some(tier_cfg) = map.get(req) {
                if let Some(primary) = tier_cfg.get("primary").and_then(|v| v.as_str()) {
                    if !primary.is_empty() {
                        return primary.to_string();
                    }
                }
            }
        }

        "gemini-2.5-flash".to_string()
    }

    /// 组装发送给 Gemini 的标准请求
    pub async fn build_inference_request(&self, user_input: &str, max_tokens: usize) -> GenerateRequest {
        // 1. 获取折叠后的历史记忆
        let history = self.memory.get_folded_context(max_tokens);
        
        let mut contents = Vec::new();
        
        // 2. 将历史对话转换为 Gemini API 格式
        for msg in history {
            let mut parts = Vec::new();
            for content in &msg.contents {
                if let crate::core::interfaces::MessageContent::Text(text) = content {
                    parts.push(Part {
                        text: Some(text.clone()),
                        function_call: None,
                        function_response: None,
                    });
                }
            }
            contents.push(Content {
                role: msg.role.clone(),
                parts,
            });
        }
        
        // 3. 追加当前用户的输入
        contents.push(Content {
            role: "user".to_string(),
            parts: vec![Part {
                text: Some(user_input.to_string()),
                function_call: None,
                function_response: None,
            }],
        });

        // 4. 构建 System Instruction
        let system_instruction = Content {
            role: "system".to_string(),
            parts: vec![Part {
                text: Some(format!(
                    "You are a specialized AI agent.\nPersona: {}\nTone: {}\nAllowed Skills: {:?}", 
                    self.persona.base_prompt, 
                    self.persona.tone, 
                    self.persona.allowed_skills
                )),
                function_call: None,
                function_response: None,
            }],
        };

        // 5. 调用 GraphSkillRegistry 获取相关技能 Schema (Tool Calling)，并依据 Persona allowed_skills 进行白名单校验
        let mut tools_payload = None;
        if let Ok(skills) = self.skill_registry.retrieve_relevant_skills(user_input, 5).await {
            let mut function_declarations = Vec::new();
            for skill in skills {
                let skill_name = skill.name().to_string();
                // 如果 Persona 声明了特定许可技能列表，则过滤非白名单技能
                if !self.persona.allowed_skills.is_empty() && !self.persona.allowed_skills.contains(&skill_name) {
                    continue;
                }
                let mut schema_val = skill.schema();
                sanitize_gemini_schema(&mut schema_val);
                if let Ok(func_dec) = serde_json::from_value::<crate::gemini::types::FunctionDeclaration>(schema_val) {
                    function_declarations.push(func_dec);
                }
            }
            if !function_declarations.is_empty() {
                tools_payload = Some(vec![Tool {
                    function_declarations,
                }]);
            }
        }

        GenerateRequest {
            contents,
            system_instruction: Some(system_instruction),
            tools: tools_payload,
        }
    }
}

/// 防御性清洗发送给 Gemini API 的 FunctionDeclaration Schema，防止非标 Array type 或缺失 items 导致 HTTP 400
fn sanitize_gemini_schema(val: &mut serde_json::Value) {
    if let serde_json::Value::Object(map) = val {
        if let Some(type_val) = map.get_mut("type") {
            if let serde_json::Value::Array(arr) = type_val {
                if let Some(first) = arr.first().and_then(|v| v.as_str()) {
                    *type_val = serde_json::Value::String(first.to_string());
                } else {
                    *type_val = serde_json::Value::String("string".to_string());
                }
            }
            if let Some(t_str) = type_val.as_str() {
                if t_str.eq_ignore_ascii_case("array") && !map.contains_key("items") {
                    map.insert(
                        "items".to_string(),
                        serde_json::json!({ "type": "string" }),
                    );
                }
            }
        }
        for (_, child) in map.iter_mut() {
            sanitize_gemini_schema(child);
        }
    } else if let serde_json::Value::Array(arr) = val {
        for child in arr.iter_mut() {
            sanitize_gemini_schema(child);
        }
    }
}
