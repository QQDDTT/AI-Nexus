use crate::utils::errors::AiNexusError;
use crate::skill::sandbox::WasmSandbox;
use serde_json::Value;
use std::sync::Arc;
use crate::core::interfaces::Skill;
use async_trait::async_trait;

/// 技能执行的流水线统筹器
pub struct SkillPipeline {
    sandbox: Arc<WasmSandbox>,
}

impl SkillPipeline {
    pub fn new(sandbox: Arc<WasmSandbox>) -> Self {
        Self { sandbox }
    }

    /// 执行一段 WASM 技能
    /// 通过内存 IPC 将 JSON 参数传入沙箱并读取返回值
    pub async fn run_skill(&self, skill_id: &str, wasm_bytes: &[u8], params: Value, fuel: u64) -> Result<Value, AiNexusError> {
        tracing::debug!("Pipeline starting execution for skill: {}", skill_id);
        
        // 调用沙箱
        let result = self.sandbox.execute_wasm(wasm_bytes, params, fuel).await.map_err(|e| {
            tracing::error!("Sandbox execution failed for skill {}: {:?}", skill_id, e);
            e
        })?;

        tracing::debug!("Pipeline execution successful for skill: {}", skill_id);
        Ok(result)
    }
}

/// 实现了 Skill trait 的动态 WASM 包装器，现作为全局执行工具
pub struct DynamicWasmSandboxTool {
    pipeline: Arc<SkillPipeline>,
}

impl DynamicWasmSandboxTool {
    pub fn new(pipeline: Arc<SkillPipeline>) -> Self {
        Self { pipeline }
    }
}

#[async_trait]
impl Skill for DynamicWasmSandboxTool {
    fn name(&self) -> &str {
        "dynamic_wasm_sandbox"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "name": "dynamic_wasm_sandbox",
            "description": "Provides a Wasmtime runtime to safely execute .wasm bytecode.",
            "parameters": {
                "type": "object",
                "properties": {
                    "script_path": {
                        "type": "string",
                        "description": "Physical path to the Wasm module, e.g. './data/generated/my_logic.wasm'."
                    },
                    "function_name": {
                        "type": "string",
                        "description": "Function name to call, default '_start'."
                    },
                    "args": {
                        "type": "array",
                        "description": "Arguments to pass to the function.",
                        "items": {
                            "type": "string"
                        }
                    },
                    "env_vars": {
                        "type": "object",
                        "description": "Environment variables."
                    }
                },
                "required": ["script_path"]
            }
        })
    }

    async fn execute(&self, params: Value) -> Result<Value, AiNexusError> {
        let script_path = params.get("script_path").and_then(|v| v.as_str()).unwrap_or_default();
        if script_path.is_empty() {
            return Err(AiNexusError::General("script_path is required".to_string()));
        }

        let wasm_bytes = tokio::fs::read(script_path).await.map_err(|e| {
            AiNexusError::General(format!("Failed to read WASM script at {}: {}", script_path, e))
        })?;

        self.pipeline.run_skill("dynamic_wasm_sandbox", &wasm_bytes, params, 10_000_000).await
    }
}

/// 组合宏技能 (MetaSkill)，能够将多个原子 Skill 串联执行
pub struct MetaSkill {
    name: String,
    description: String,
    steps: Vec<Box<dyn Skill + Send + Sync>>,
}

impl MetaSkill {
    pub fn new(name: String, description: String, steps: Vec<Box<dyn Skill + Send + Sync>>) -> Self {
        Self { name, description, steps }
    }
}

#[async_trait]
impl Skill for MetaSkill {
    fn name(&self) -> &str {
        &self.name
    }

    fn schema(&self) -> Value {
        // 组合技能的对外 Schema 默认取决于它的第一个步骤，但覆盖其名称和描述
        if let Some(first_step) = self.steps.first() {
            let mut s = first_step.schema();
            if let Some(obj) = s.as_object_mut() {
                obj.insert("name".to_string(), serde_json::json!(self.name));
                obj.insert("description".to_string(), serde_json::json!(self.description));
            }
            s
        } else {
            serde_json::json!({
                "name": self.name,
                "description": self.description
            })
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, AiNexusError> {
        if self.steps.is_empty() {
            return Err(AiNexusError::General(format!("MetaSkill '{}' has no steps to execute", self.name)));
        }

        let mut current_payload = params;
        
        for (i, step) in self.steps.iter().enumerate() {
            tracing::debug!("MetaSkill '{}' executing step {}/{} [{}]", self.name, i + 1, self.steps.len(), step.name());
            // 将上一步的输出全量抛给下一步作为输入
            current_payload = step.execute(current_payload).await?;
            
            // 如果内部抛出错误，直接被 `?` 向上熔断
        }

        tracing::info!("MetaSkill '{}' completed all {} steps successfully.", self.name, self.steps.len());
        Ok(current_payload)
    }
}
