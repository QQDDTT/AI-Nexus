use ai_nexus::agent::instance::AgentInstance;
use ai_nexus::agent::memory::InMemoryStore;
use ai_nexus::storage::{Storage, LedgerBlock};
use std::time::{SystemTime, UNIX_EPOCH};
use ai_nexus::core::interfaces::{Channel, ChatMessage, MessageContent, Persona, SkillRegistry};
use ai_nexus::gemini::client::GeminiClient;
use ai_nexus::gemini::embedding::GeminiEmbeddingClient;
use ai_nexus::os::channel::LocalTerminalChannel;
use ai_nexus::os::telegram::TelegramChannel;
use ai_nexus::agent::meta::MetaAgent;
use ai_nexus::skill::sandbox::WasmSandbox;
use ai_nexus::skill::pipeline::SkillPipeline;
use ai_nexus::skill::registry::GraphSkillRegistry;
use ai_nexus::iam::IamGateway;
use std::sync::Arc;
use std::env;
use tracing::{error, info, Level};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 0. 加载 .env 环境变量
    dotenvy::dotenv().ok();

    // 1. 初始化终端日志输出格式
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("=== AI-Nexus Terminal MVP ===");

    // 2. 检查 API Key
    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            error!("Environment variable GEMINI_API_KEY not found!");
            info!("Please run: export GEMINI_API_KEY='your_key_here' or add it to .env");
            return Ok(());
        }
    };

    // 3. 组装三大核心模块
    let gemini_client = Arc::new(GeminiClient::new(api_key.clone()));
    let embedding_client = Arc::new(GeminiEmbeddingClient::new(api_key.clone()));
    
    // 初始化 Storage 层
    let config = ai_nexus::core::config::get_config();
    let storage = Arc::new(Storage::new(&config.system.storage_path)?);
    info!("Storage layer initialized.");

    // 初始化 WASM 技能沙箱流水线
    let wasm_sandbox = Arc::new(WasmSandbox::new()?);
    let skill_pipeline = Arc::new(SkillPipeline::new(wasm_sandbox));
    info!("WASM Skill Sandbox initialized.");

    // 初始化 IAM 安全审计网关
    let iam_gateway = Arc::new(IamGateway::new(storage.nexus_db.clone()));
    info!("IAM Gateway initialized.");

    // 初始化 GraphSkillRegistry
    let skill_registry = Arc::new(GraphSkillRegistry::new(
        storage.graph_store.clone(),
        storage.vector_store.clone(),
        embedding_client.clone(),
    ));

    // 注入原生底层技能 (Native Skills)
    if let Err(e) = skill_registry.register_skill(Box::new(ai_nexus::skill::native::WebSearchSkill::new())).await {
        error!("Failed to register WebSearchSkill: {}", e);
    }
    if let Err(e) = skill_registry.register_skill(Box::new(ai_nexus::skill::native::FileGenerateSkill)).await {
        error!("Failed to register FileGenerateSkill: {}", e);
    }
    let sandbox_tool = ai_nexus::skill::pipeline::DynamicWasmSandboxTool::new(skill_pipeline.clone());
    if let Err(e) = skill_registry.register_skill(Box::new(sandbox_tool)).await {
        error!("Failed to register DynamicWasmSandboxTool: {}", e);
    }
    info!("Native skills registered.");

    // 启动 Dashboard API 存根 (后台任务)
    let app_state = ai_nexus::os::api::AppState {
        storage: storage.clone(),
        skill_registry: skill_registry.clone(),
        gemini_client: gemini_client.clone(),
    };
    tokio::spawn(async move {
        ai_nexus::os::api::start_server(app_state).await;
    });

    // 等待 Dashboard API 存根完成数据 Seed
    let mut persona_value = None;
    for _ in 0..10 {
        if let Some(agent_data) = storage.nexus_db.collection("agents").get(&config.system.default_agent_id) {
            if let Ok(agent_def) = serde_json::from_value::<ai_nexus::core::interfaces::AgentDef>(agent_data) {
                if let Some(p) = agent_def.metadata.get("persona") {
                    persona_value = Some(p.clone());
                    break;
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    
    let persona: Persona = if let Some(pv) = persona_value {
        serde_json::from_value(pv).unwrap_or_else(|_| Persona {
            base_prompt: "Fallback Developer".to_string(),
            allowed_skills: vec![],
            tone: "neutral".to_string(),
        })
    } else {
        Persona {
            base_prompt: "Fallback Developer".to_string(),
            allowed_skills: vec![],
            tone: "neutral".to_string(),
        }
    };

    let mut agent = AgentInstance::new(
        config.system.default_agent_id.clone(),
        config.system.default_admin_id.clone(),
        "Tier-1-Logic".to_string(),
        persona,
        Box::new(InMemoryStore::new(
            storage.graph_store.clone(),
            storage.vector_store.clone(),
            embedding_client.clone(),
        )),
        skill_registry.clone(),
    );

    info!("Agent [{}] initialized and ready.", config.system.default_agent_id);

    // 4. 多通道接收器架构
    let (main_tx, mut main_rx) = mpsc::unbounded_channel::<(String, String, String)>();

    let local_channel = Arc::new(LocalTerminalChannel::new(config.system.default_admin_id.clone()));
    let local_tx = main_tx.clone();
    let lc_clone = local_channel.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(inputs) = lc_clone.receive_input().await {
                if let Some(MessageContent::Text(t)) = inputs.first() {
                    let _ = local_tx.send(("LocalTerminal".to_string(), "local_admin_001".to_string(), t.clone()));
                }
            }
        }
    });

    let mut telegram_channel_opt = None;
    if let Ok(token) = env::var("TELEGRAM_BOT_TOKEN") {
        let telegram_channel = Arc::new(TelegramChannel::new(&token));
        telegram_channel_opt = Some(telegram_channel.clone());
        let tg_clone = telegram_channel.clone();
        let tg_tx = main_tx.clone();
        info!("Telegram Channel initialized.");
        tokio::spawn(async move {
            loop {
                if let Ok(inputs) = tg_clone.receive_input().await {
                    if let Some(MessageContent::Text(t)) = inputs.first() {
                        let parts: Vec<&str> = t.splitn(2, '|').collect();
                        if parts.len() == 2 {
                            let _ = tg_tx.send(("Telegram".to_string(), parts[0].to_string(), parts[1].to_string()));
                        }
                    }
                }
            }
        });
    } else {
        info!("TELEGRAM_BOT_TOKEN not found, skipping Telegram channel.");
    }

    // 辅助回复函数
    let send_reply = |channel_name: &str, user_id: &str, msg: String| {
        let lc = local_channel.clone();
        let tc = telegram_channel_opt.clone();
        let c_name = channel_name.to_string();
        let u_id = user_id.to_string();
        tokio::spawn(async move {
            if c_name == "LocalTerminal" {
                let _ = lc.send_reply(&u_id, vec![MessageContent::Text(msg)]).await;
            } else if c_name == "Telegram" {
                if let Some(tg) = tc {
                    let _ = tg.send_reply(&u_id, vec![MessageContent::Text(msg)]).await;
                }
            }
        });
    };

    // 5. 开启集中事件循环
    while let Some((source_channel, user_id, user_text)) = main_rx.recv().await {
        if user_text.trim() == "/exit" && source_channel == "LocalTerminal" {
            info!("Exiting AI-Nexus terminal...");
            break;
        }

        if user_text.trim().starts_with("/create_skill") {
            let prompt = user_text.trim().trim_start_matches("/create_skill").trim();
            if prompt.is_empty() {
                send_reply(&source_channel, &user_id, "Usage: /create_skill <description>".to_string());
                continue;
            }
            send_reply(&source_channel, &user_id, format!("MetaAgent is generating skill for: {}", prompt));
            
            let meta_agent = MetaAgent::new(
                gemini_client.clone(),
                config.system.meta_workspace.clone(),
                config.models.global_gemini_model.clone(),
            );

            match meta_agent.generate_and_compile_skill(prompt).await {
                Ok(wasm_bytes) => {
                    let path = format!("{}/latest_skill.wasm", config.system.storage_path);
                    if let Err(e) = std::fs::write(&path, &wasm_bytes) {
                        send_reply(&source_channel, &user_id, format!("Failed to write skill: {}", e));
                        continue;
                    }
                    
                    send_reply(&source_channel, &user_id, format!("Skill successfully compiled! Saved to {}. You can use dynamic_wasm_sandbox to execute it.", path));
                }
                Err(e) => {
                    send_reply(&source_channel, &user_id, format!("MetaAgent Failed: {}", e));
                }
            }
            continue;
        }

        if user_text.trim() == "/stats" {
            let records: Vec<LedgerBlock> = storage.ledger_store.read_all_records().unwrap_or_default();
            let total_input: u32 = records.iter().map(|r| r.input_tokens).sum();
            let total_output: u32 = records.iter().map(|r| r.output_tokens).sum();
            let total_cost: f64 = records.iter().map(|r| r.est_cost_usd).sum();
            send_reply(&source_channel, &user_id, format!(
                "=== Binary Ledger Stats ===\nTotal Calls: {}\nTotal Input Tokens: {}\nTotal Output Tokens: {}\nEstimated Cost: ${:.5}\n===========================",
                records.len(), total_input, total_output, total_cost
            ));
            continue;
        }

        if user_text.trim().is_empty() {
            continue;
        }

        // [Awake & Auth] 拦截与鉴权
        if let Err(e) = iam_gateway.verify_and_deduct_quota(&user_id, 500).await {
            send_reply(&source_channel, &user_id, format!("⚠️ [IAM Blocked]: {}", e));
            continue;
        }

        // B. Agent 整合上下文与设定，生成请求体
        let mut request = agent.build_inference_request(&user_text, 1024).await;
        
        let mut final_reply = String::new();
        let mut total_in_tokens = 0;
        let mut total_out_tokens = 0;

        // Function calling 循环执行 (最大深度 5 层)
        for _ in 0..5 {
            let routing_val = storage.nexus_db.collection("settings").get("capability_routing");
            let target_model = agent.resolve_target_model(routing_val.as_ref());
            info!("Sending request via Model Router to target model: {}...", target_model);
            match gemini_client.generate_content(&target_model, &request).await {
                Ok(response) => {
                    let in_t = response.usage_metadata.as_ref().map_or(0, |u| u.prompt_token_count);
                    let out_t = response.usage_metadata.as_ref().map_or(0, |u| u.candidates_token_count);
                    total_in_tokens += in_t;
                    total_out_tokens += out_t;

                    if let Some(candidates) = response.candidates {
                        if let Some(candidate) = candidates.first() {
                            if let Some(part) = candidate.content.parts.first() {
                                if let Some(fc) = &part.function_call {
                                    info!("Gemini invoked skill: {}", fc.name);
                                    let mut execution_result = serde_json::json!({"error": "Skill not found in registry"});
                                    
                                    if let Some(skill) = agent.skill_registry.get_skill(&fc.name).await {
                                        match skill.execute(fc.args.clone()).await {
                                            Ok(res) => {
                                                execution_result = res;
                                                info!("Skill '{}' executed successfully.", fc.name);
                                            }
                                            Err(e) => {
                                                execution_result = serde_json::json!({"error": format!("Execution failed: {}", e)});
                                                tracing::error!("Skill '{}' failed: {}", fc.name, e);
                                            }
                                        }
                                    }

                                    // 将模型发出的 function_call 加入上下文
                                    request.contents.push(candidate.content.clone());
                                    
                                    // 将函数执行结果作为 function_response 喂给模型
                                    request.contents.push(ai_nexus::gemini::types::Content {
                                        role: "user".to_string(), // 在 Gemini 中 response 通常属于 user
                                        parts: vec![ai_nexus::gemini::types::Part {
                                            text: None,
                                            function_call: None,
                                            function_response: Some(ai_nexus::gemini::types::FunctionResponse {
                                                name: fc.name.clone(),
                                                response: execution_result,
                                            }),
                                        }],
                                    });
                                    continue; // 带着结果再次请求模型
                                } else if let Some(text) = &part.text {
                                    final_reply = text.clone();
                                    break; // 获取到最终文本，退出循环
                                }
                            }
                        }
                    }
                    final_reply = "[Empty or unhandled response]".to_string();
                    break;
                }
                Err(e) => {
                    tracing::error!("Gemini API Failed: {:?}", e);
                    final_reply = format!("Error communicating with Gemini: {}", e);
                    break;
                }
            }
        }

        // 记录到二进制 Ledger
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let ledger_record = LedgerBlock {
            timestamp: now,
            user_id: user_id.clone(),
            model: config.models.global_gemini_model.clone(),
            input_tokens: total_in_tokens as u32,
            output_tokens: total_out_tokens as u32,
            est_cost_usd: (total_in_tokens as f64 * 0.075 / 1_000_000.0) + (total_out_tokens as f64 * 0.3 / 1_000_000.0),
        };
        if let Err(e) = storage.ledger_store.append_record(&ledger_record) {
            error!("Failed to append to binary ledger: {:?}", e);
        }

        // D. 归档双方对话到记忆中
        agent.memory.push_short_term(ChatMessage {
            role: "user".to_string(),
            contents: vec![MessageContent::Text(user_text.clone())],
        });
        agent.memory.push_short_term(ChatMessage {
            role: "model".to_string(),
            contents: vec![MessageContent::Text(final_reply.clone())],
        });

        // D2. 异步抛写状态快照以供微秒级复活
        let history = agent.memory.get_folded_context(100000);
        ai_nexus::storage::snapshot::StateSnapshot::save_context_state(
            &user_id,
            &source_channel,
            history,
            storage.session_store.clone()
        ).await;

        // E. 通过 Channel 将结果打印回原渠道
        send_reply(&source_channel, &user_id, final_reply);
    }

    Ok(())
}
