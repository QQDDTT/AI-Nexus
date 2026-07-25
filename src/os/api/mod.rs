use crate::core::interfaces::{ProviderDef, PersonaDef, AgentDef, GatewayDef, TriggerDef, SettingsDef};
use axum::{
    extract::{FromRef, Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::storage::nexus_db::NexusDb;
use crate::skill::registry::GraphSkillRegistry;
use tokio::net::TcpListener;

use crate::storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
    pub skill_registry: Arc<GraphSkillRegistry>,
    pub gemini_client: Arc<crate::gemini::client::GeminiClient>,
}

impl FromRef<AppState> for Arc<NexusDb> {
    fn from_ref(app_state: &AppState) -> Arc<NexusDb> {
        app_state.storage.nexus_db.clone()
    }
}

impl FromRef<AppState> for Arc<Storage> {
    fn from_ref(app_state: &AppState) -> Arc<Storage> {
        app_state.storage.clone()
    }
}

#[derive(Deserialize)]
struct LoginRequest {
    username: Option<String>,
    password: Option<String>,
} 

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

async fn login(Json(payload): Json<LoginRequest>) -> Result<Json<LoginResponse>, StatusCode> {
    if payload.username.as_deref() == Some(crate::core::constants::DEFAULT_ADMIN_USERNAME) 
        && payload.password.as_deref() == Some(crate::core::constants::DEFAULT_ADMIN_PASSWORD) {
        Ok(Json(LoginResponse {
            token: crate::core::constants::STUB_AUTH_TOKEN.to_string(),
        }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn check_auth(headers: &HeaderMap) -> Result<(), StatusCode> {
    if let Some(auth_header) = headers.get("Authorization") {
        if auth_header.to_str().unwrap_or("").starts_with(crate::core::constants::AUTH_TOKEN_PREFIX) {
            return Ok(());
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

async fn get_stats(State(storage): State<Arc<crate::storage::Storage>>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let db = &storage.nexus_db;
    let gateways: Vec<Value> = db.collection("gateways").iter().into_iter().map(|(_, v)| v).collect();
    
    let mut real_agents = Vec::new();
    for entry in db.collection("agents").iter() {
        let mut agent = entry.1.clone();
        if agent.get("uptime").is_none() {
            agent["uptime"] = json!("0m");
            agent["tasks_completed"] = json!(0);
        }
        real_agents.push(agent);
    }
    
    let active_sessions = storage.sessions.len();
    
    let mut total_tokens: u64 = 0;
    if let Ok(records) = storage.ledger_store.read_all_records::<crate::storage::models::LedgerBlock>() {
        for record in records {
            total_tokens += record.input_tokens as u64 + record.output_tokens as u64;
        }
    }
    let total_tokens_str = if total_tokens > 1_000_000 {
        format!("{:.1}M", total_tokens as f64 / 1_000_000.0)
    } else if total_tokens > 1_000 {
        format!("{:.1}K", total_tokens as f64 / 1_000.0)
    } else {
        total_tokens.to_string()
    };
    
    let mut skills_usage = Vec::new();
    for entry in db.collection("skills").iter() {
        if let Some(name) = entry.1.get("name").and_then(|n| n.as_str()) {
            skills_usage.push(json!({
                "skill": name,
                "calls": 0,
                "success_rate": 100.0
            }));
        }
    }

    Ok(Json(json!({
        "active_sessions": active_sessions,
        "active_sessions_trend": "Realtime",
        "total_tokens": total_tokens_str,
        "total_tokens_trend": "Recorded",
        "api_health": "100%",
        "api_health_trend": "NexusDB Healthy",
        "gateways": gateways,
        "agents": real_agents,
        "skills_usage": skills_usage
    })))
}

async fn get_token_trend(State(storage): State<Arc<crate::storage::Storage>>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let mut daily_tokens: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
    if let Ok(records) = storage.ledger_store.read_all_records::<crate::storage::models::LedgerBlock>() {
        for record in records {
            let day = record.timestamp / 86400;
            *daily_tokens.entry(day).or_insert(0) += record.input_tokens as u64 + record.output_tokens as u64;
        }
    }
    
    if daily_tokens.is_empty() {
        return Ok(Json(json!({ "trend": [] })));
    }

    let mut trend = Vec::new();
    let mut days: Vec<_> = daily_tokens.keys().cloned().collect();
    days.sort();
    for day in days {
        trend.push(json!({
            "name": format!("Day {}", day % 365),
            "tokens": daily_tokens[&day]
        }));
    }

    Ok(Json(json!({ "trend": trend })))
}

async fn get_gateways(State(db): State<Arc<NexusDb>>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let gateways = db.collection("gateways").iter().into_iter().map(|(_, v)| v).collect::<Vec<_>>();
    Ok(Json(json!(gateways)))
}

async fn toggle_gateway(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    if let Some(mut gateway) = db.collection("gateways").get(&id) {
        let new_status = if let Some(status) = gateway.get("status").and_then(|s| s.as_str()) {
            if status == "Idle" { "Active" } else { "Idle" }
        } else {
            "Active"
        };
        gateway["status"] = json!(new_status);
        db.insert("gateways", &id, gateway.clone()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        return Ok(Json(json!({ "status": "ok", "id": id, "new_state": new_status })));
    }
    Err(StatusCode::NOT_FOUND)
}

async fn create_gateway(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Json(payload): Json<GatewayDef>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let id = &payload.id;
    let val = serde_json::to_value(&payload).unwrap();
    let _ = db.insert("gateways", id, val.clone());
    return Ok(Json(json!({ "status": "created", "gateway": val })));
}

async fn delete_gateway(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let _ = db.delete("gateways", &id);
    Ok(Json(json!({ "status": "deleted", "id": id })))
}

async fn config_gateway(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Path(id): Path<String>, Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    if let Some(mut gateway) = db.collection("gateways").get(&id) {
        if let Some(bound_persona) = payload.get("bound_persona").and_then(|v| v.as_str()) {
            if !bound_persona.is_empty() {
                gateway["bound_persona"] = json!(bound_persona);
            } else if let Some(obj) = gateway.as_object_mut() {
                obj.remove("bound_persona");
            }
        }
        if let Some(key) = payload.get("key").and_then(|v| v.as_str()) {
            gateway["key"] = json!(key);
        }
        db.insert("gateways", &id, gateway.clone()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        return Ok(Json(json!({ "status": "ok", "id": id })));
    }
    Err(StatusCode::NOT_FOUND)
}

async fn get_settings(State(db): State<Arc<NexusDb>>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let settings = db.collection("settings").iter().into_iter().collect::<serde_json::Map<_, _>>();
    Ok(Json(Value::Object(settings)))
}

async fn update_settings(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Json(payload): Json<SettingsDef>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let val = serde_json::to_value(&payload).unwrap();
    if let serde_json::Value::Object(map) = val.clone() {
        for (k, v) in map {
            db.insert("settings", &k, v).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }
    Ok(Json(serde_json::json!({ "status": "updated", "settings": val })))
}

async fn delete_session(headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    Ok(Json(json!({ "status": "deleted", "id": id })))
}

async fn get_ledger(State(storage): State<Arc<crate::storage::Storage>>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let mut list = Vec::new();
    if let Ok(records) = storage.ledger_store.read_all_records::<crate::storage::models::LedgerBlock>() {
        for record in records {
            list.push(json!({
                "time": record.timestamp,
                "user_id": record.user_id,
                "model": record.model,
                "input_tokens": record.input_tokens,
                "output_tokens": record.output_tokens,
                "est_cost_usd": record.est_cost_usd
            }));
        }
    }
    Ok(Json(json!(list)))
}

// --- Model Router Endpoints ---
async fn get_model_routing(State(db): State<Arc<NexusDb>>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    
    // Check if there's a stored capability routing map
    if let Some(routing) = db.collection("settings").get("capability_routing") {
        return Ok(Json(routing));
    }
    
    // Default Routing Table if none exists
    Ok(Json(crate::core::constants::DEFAULT_CAPABILITY_ROUTING.clone()))
}

async fn update_model_routing(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    db.insert("settings", "capability_routing", payload.clone()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "status": "updated", "routing": payload })))
}

async fn get_providers(State(db): State<Arc<NexusDb>>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let mut providers: Vec<Value> = db.collection("providers").iter().into_iter().map(|(_, v)| v).collect();
    
    // Mask API Keys for frontend display
    for provider in &mut providers {
        if let Some(obj) = provider.as_object_mut() {
            if let Some(key) = obj.get("api_key").and_then(|k| k.as_str()) {
                if key.len() > 6 {
                    obj.insert("api_key".to_string(), json!(format!("{}****", &key[0..6])));
                } else {
                    obj.insert("api_key".to_string(), json!("********"));
                }
            }
        }
    }
    
    Ok(Json(json!(providers)))
}

async fn add_provider(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Json(payload): Json<ProviderDef>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let val = serde_json::to_value(&payload).unwrap();
    let _ = db.insert("providers", &payload.id, val.clone());
    return Ok(Json(json!({ "status": "added", "provider": val })));
}

async fn delete_provider(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    if db.collection("providers").get(&id).is_some() {
        db.delete("providers", &id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(json!({ "status": "deleted", "id": id })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn toggle_failover(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Path(name): Path<String>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    if let Some(mut fallback) = db.collection("fallbacks").get(&name) {
        let new_status = if fallback.get("status").and_then(|s| s.as_str()) == Some("Active") { "Standby" } else { "Active" };
        fallback["status"] = json!(new_status);
        let _ = db.insert("fallbacks", &name, fallback);
        return Ok(Json(json!({ "status": "ok", "name": name, "new_state": new_status })));
    }
    Err(StatusCode::NOT_FOUND)
}

// --- Sessions Endpoints ---
async fn get_sessions(State(storage): State<Arc<crate::storage::Storage>>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let config = crate::core::config::get_config();
    let mut list = Vec::new();
    for entry in storage.sessions.iter() {
        let (id, _session) = (entry.key(), entry.value());
        list.push(json!({
            "session_id": id,
            "source": "Active Session",
            "model": config.models.global_gemini_model.clone(),
            "tokens": 0,
            "status": "Active"
        }));
    }
    Ok(Json(json!(list)))
}

// --- Skills Endpoints ---
async fn get_skills(State(_state): State<AppState>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let mut skills: Vec<Value> = Vec::new();

    // Add Markdown skills from the `core_skills/` directory (System/Core Skills)
    if let Ok(entries) = std::fs::read_dir("core_skills") {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let md_path = entry.path().join("SKILL.md");
                    if md_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&md_path) {
                            if !skills.iter().any(|v| v.get("name").and_then(|n| n.as_str()) == Some(&name)) {
                                skills.push(json!({
                                    "id": name.clone(),
                                    "name": name,
                                    "status": "Active",
                                    "type": "Markdown",
                                    "is_core": true,
                                    "source_code": content
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    // Add Markdown skills from the `skills/` directory (User Skills)
    if let Ok(entries) = std::fs::read_dir("skills") {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let md_path = entry.path().join("SKILL.md");
                    if md_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&md_path) {
                            if !skills.iter().any(|v| v.get("name").and_then(|n| n.as_str()) == Some(&name)) {
                                skills.push(json!({
                                    "id": name.clone(),
                                    "name": name,
                                    "status": "Active",
                                    "type": "Markdown",
                                    "is_core": false,
                                    "source_code": content
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Json(json!(skills)))
}

#[derive(serde::Deserialize)]
struct CompileSkillPayload {
    name: String,
    source_code: String,
}

async fn compile_skill(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Json(payload): Json<CompileSkillPayload>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    
    let workspace_src = "src/ainexus-test/meta-workspace/src/lib.rs";
    if let Err(e) = std::fs::write(workspace_src, &payload.source_code) {
        return Ok(Json(json!({"error": format!("Failed to write source: {}", e)})));
    }
    
    // Run cargo build
    let output = std::process::Command::new("cargo")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .current_dir("src/ainexus-test/meta-workspace")
        .output();
        
    match output {
        Ok(out) if out.status.success() => {
            let compiled_wasm = "src/ainexus-test/meta-workspace/target/wasm32-unknown-unknown/release/meta_workspace.wasm";
            let dest_wasm = format!("data/blocks/{}.wasm", payload.name);
            
            if let Err(e) = std::fs::copy(compiled_wasm, dest_wasm) {
                return Ok(Json(json!({"error": format!("Failed to copy wasm: {}", e)})));
            }
            
            // Register skill in DB
            let skill_data = json!({
                "id": payload.name.clone(),
                "name": payload.name.clone(),
                "status": "Active",
                "source_code": payload.source_code,
            });
            let _ = db.insert("skills", &payload.name, skill_data);
            
            Ok(Json(json!({"status": "success"})))
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Ok(Json(json!({"error": format!("Compilation failed: {}", stderr)})))
        }
        Err(e) => {
            Ok(Json(json!({"error": format!("Failed to execute cargo: {}", e)})))
        }
    }
}

async fn delete_skill(State(storage): State<Arc<crate::storage::Storage>>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    
    let core_md_path = format!("core_skills/{}/SKILL.md", id);
    if std::path::Path::new(&core_md_path).exists() {
        return Ok(Json(json!({ "error": "Cannot delete core skills." })));
    }
    
    let db = &storage.nexus_db;
    let _ = db.delete("skills", &id);
    let _ = std::fs::remove_file(format!("data/blocks/{}.wasm", id));
    let _ = std::fs::remove_dir_all(format!("skills/{}", id));
    Ok(Json(json!({ "status": "deleted", "id": id })))
}

async fn save_markdown_skill(headers: HeaderMap, Json(payload): Json<CompileSkillPayload>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    
    // Deny modifications to core_skills
    let core_md_path = format!("core_skills/{}/SKILL.md", payload.name);
    if std::path::Path::new(&core_md_path).exists() {
        return Ok(Json(json!({"error": "Cannot save core skills."})));
    }
    
    // Construct the path: skills/{name}/SKILL.md
    let md_path = format!("skills/{}/SKILL.md", payload.name);
    let path = std::path::Path::new(&md_path);
    
    if !path.exists() {
        return Ok(Json(json!({"error": format!("Markdown file for skill {} not found in user skills", payload.name)})));
    }
    
    if let Err(e) = std::fs::write(path, &payload.source_code) {
        return Ok(Json(json!({"error": format!("Failed to write markdown: {}", e)})));
    }
    
    Ok(Json(json!({"status": "success"})))
}

#[derive(serde::Deserialize)]
struct AiAssistPayload {
    skill_name: String,
    current_code: String,
    instruction: String,
}

async fn ai_assist(State(state): State<AppState>, headers: HeaderMap, Json(payload): Json<AiAssistPayload>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    tracing::info!("AI Assist triggered for skill: {}", payload.skill_name);

    let meta_skill_path = "core_skills/meta_skill/SKILL.md";
    let meta_skill_content = std::fs::read_to_string(meta_skill_path)
        .unwrap_or_else(|_| "You are an AI assistant helping to write a Rust WASM skill.".to_string());

    let prompt = format!(
        "User Instruction: {}\n\nCurrent Code:\n```\n{}\n```\n\nPlease provide the COMPLETE updated code based on the user instruction. Reply ONLY with the updated code block, no other explanations.",
        payload.instruction,
        payload.current_code
    );

    let request = crate::gemini::types::GenerateRequest {
        contents: vec![crate::gemini::types::Content {
            role: "user".to_string(),
            parts: vec![crate::gemini::types::Part {
                text: Some(prompt),
                function_call: None,
                function_response: None,
            }],
        }],
        system_instruction: Some(crate::gemini::types::Content {
            role: "system".to_string(),
            parts: vec![crate::gemini::types::Part {
                text: Some(meta_skill_content),
                function_call: None,
                function_response: None,
            }],
        }),
        tools: None,
    };

    let model = crate::core::config::get_config().models.global_gemini_model.clone();
    match state.gemini_client.generate_content(&model, &request).await {
        Ok(response) => {
            if let Some(candidates) = response.candidates {
                if let Some(candidate) = candidates.first() {
                    if let Some(part) = candidate.content.parts.first() {
                        if let Some(text) = &part.text {
                            let mut code = text.trim();
                            if code.starts_with("```rust") {
                                code = code.strip_prefix("```rust").unwrap().trim();
                                if code.ends_with("```") {
                                    code = code.strip_suffix("```").unwrap().trim();
                                }
                            } else if code.starts_with("```markdown") {
                                code = code.strip_prefix("```markdown").unwrap().trim();
                                if code.ends_with("```") {
                                    code = code.strip_suffix("```").unwrap().trim();
                                }
                            } else if code.starts_with("```") {
                                code = code.strip_prefix("```").unwrap().trim();
                                if code.ends_with("```") {
                                    code = code.strip_suffix("```").unwrap().trim();
                                }
                            }
                            return Ok(Json(json!({"status": "success", "suggested_code": code})));
                        }
                    }
                }
            }
            Ok(Json(json!({"error": "Empty response from Gemini"})))
        },
        Err(e) => Ok(Json(json!({"error": format!("{:?}", e)})))
    }
}

async fn toggle_skill(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    if let Some(mut skill) = db.collection("skills").get(&id) {
        let new_status = if skill.get("status").and_then(|s| s.as_str()) == Some("Active") { "Disabled" } else { "Active" };
        skill["status"] = json!(new_status);
        let _ = db.insert("skills", &id, skill);
        return Ok(Json(json!({ "status": "ok", "id": id, "new_state": new_status })));
    }
    Err(StatusCode::NOT_FOUND)
}

// --- Agents Endpoints ---
async fn get_agents(State(db): State<Arc<NexusDb>>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let mut agents = db.collection("agents").iter().into_iter().map(|(_, v)| v).collect::<Vec<_>>();
    // Join persona for each agent
    for agent in &mut agents {
        if let Some(obj) = agent.as_object_mut() {
            if let Some(persona_id) = obj.get("persona_id").and_then(|p| p.as_str()) {
                if let Some(persona) = db.collection("personas").get(persona_id) {
                    obj.insert("persona".to_string(), persona);
                } else if !obj.contains_key("persona") {
                    obj.insert("persona".to_string(), serde_json::json!({
                        "id": persona_id,
                        "name": "Unknown",
                        "base_prompt": "",
                        "allowed_skills": [],
                        "tone": ""
                    }));
                }
            } else if !obj.contains_key("persona") {
               obj.insert("persona".to_string(), serde_json::json!({
                        "id": "unknown",
                        "name": "Unknown",
                        "base_prompt": "",
                        "allowed_skills": [],
                        "tone": ""
                    }));
            }
        }
    }
    Ok(Json(serde_json::json!(agents)))
}

async fn create_agent(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("agent_default").to_string();
    let _ = db.insert("agents", &id, payload.clone());
    Ok(Json(serde_json::json!({ "status": "created", "agent": payload })))
}

async fn update_agent(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Path(id): Path<String>, Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    if db.collection("agents").get(&id).is_some() {
        let _ = db.insert("agents", &id, payload.clone());
        return Ok(Json(serde_json::json!({ "status": "updated", "agent": payload })));
    }
    Err(StatusCode::NOT_FOUND)
}

async fn delete_agent(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let _ = db.delete("agents", &id);
    Ok(Json(json!({ "status": "deleted", "id": id })))
}

// --- Triggers Endpoints ---
async fn get_triggers(State(db): State<Arc<NexusDb>>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let triggers = db.collection("triggers").iter().into_iter().map(|(_, v)| v).collect::<Vec<_>>();
    Ok(Json(json!(triggers)))
}

async fn create_trigger(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Json(payload): Json<TriggerDef>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let val = serde_json::to_value(&payload).unwrap();
    let _ = db.insert("triggers", &payload.id, val.clone());
    Ok(Json(serde_json::json!({ "status": "created", "trigger": val })))
}

async fn update_trigger(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Path(id): Path<String>, Json(payload): Json<TriggerDef>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    if db.collection("triggers").get(&id).is_some() {
        let val = serde_json::to_value(&payload).unwrap();
        let _ = db.insert("triggers", &id, val.clone());
        return Ok(Json(serde_json::json!({ "status": "updated", "trigger": val })));
    }
    Err(StatusCode::NOT_FOUND)
}

async fn delete_trigger(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let _ = db.delete("triggers", &id);
    Ok(Json(json!({ "status": "deleted", "id": id })))
}

// --- Personas Endpoints ---
async fn get_personas(State(db): State<Arc<NexusDb>>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let personas = db.collection("personas").iter().into_iter().map(|(_, v)| v).collect::<Vec<_>>();
    Ok(Json(json!(personas)))
}

async fn create_persona(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Json(payload): Json<PersonaDef>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let val = serde_json::to_value(&payload).unwrap();
    let _ = db.insert("personas", &payload.id, val.clone());
    return Ok(Json(json!({ "status": "created", "persona": val })));
}

async fn update_persona(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Path(id): Path<String>, Json(payload): Json<PersonaDef>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    if db.collection("personas").get(&id).is_some() {
        let val = serde_json::to_value(&payload).unwrap();
        let _ = db.insert("personas", &id, val.clone());
        return Ok(Json(json!({ "status": "updated", "persona": val })));
    }
    Err(StatusCode::NOT_FOUND)
}

async fn delete_persona(State(db): State<Arc<NexusDb>>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    check_auth(&headers)?;
    let _ = db.delete("personas", &id);
    Ok(Json(json!({ "status": "deleted", "id": id })))
}

pub async fn start_server(app_state: AppState) {
    let nexus_db = &app_state.storage.nexus_db;
    // Seed initial database settings if empty
    if nexus_db.collection("settings").get("db_path").is_none() {
        let _ = nexus_db.insert("settings", "db_path", json!("/tmp"));
        let _ = nexus_db.insert("settings", "session_timeout_ms", json!(3000));
        let _ = nexus_db.insert("settings", "log_masking", json!(false));
    }
    
    let config = crate::core::config::get_config();
    if nexus_db.collection("agents").get(&config.system.default_agent_id).is_none() {
        let _ = nexus_db.insert("agents", &config.system.default_agent_id, json!({
            "id": config.system.default_agent_id.clone(),
            "name": "Local Admin",
            "capability_requirement": "Tier-1-Logic",
            "persona": {
                "base_prompt": "You are a helpful AI assistant.",
                "allowed_skills": [],
                "tone": "professional"
            },
            "status": "Active"
        }));
    }

    let app = Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/dashboard/stats", get(get_stats))
        .route("/api/dashboard/token-trend", get(get_token_trend))
        .route("/api/gateways", get(get_gateways).post(create_gateway))
        .route("/api/gateways/:id", delete(delete_gateway))
        .route("/api/gateways/:id/toggle", post(toggle_gateway))
        .route("/api/gateways/:id/config", axum::routing::put(config_gateway))
        .route("/api/settings", get(get_settings).put(update_settings))
        .route("/api/sessions", get(get_sessions))
        .route("/api/sessions/:id", delete(delete_session))
        .route("/api/ledger", get(get_ledger))
        .route("/api/models/routing", get(get_model_routing).put(update_model_routing))
        .route("/api/models/providers", get(get_providers))
        .route("/api/models/providers", post(add_provider))
        .route("/api/models/providers/:id", delete(delete_provider))
        .route("/api/models/failover/:name/toggle", post(toggle_failover))
        .route("/api/skills", get(get_skills))
        .route("/api/skills/compile", post(compile_skill))
        .route("/api/skills/save_md", post(save_markdown_skill))
        .route("/api/skills/ai-assist", post(ai_assist))
        .route("/api/skills/:id", delete(delete_skill))
        .route("/api/skills/:id/toggle", post(toggle_skill))
        .route("/api/agents", get(get_agents).post(create_agent))
        .route("/api/agents/:id", axum::routing::put(update_agent).delete(delete_agent))
        .route("/api/personas", get(get_personas).post(create_persona))
        .route("/api/personas/:id", axum::routing::put(update_persona).delete(delete_persona))
        .route("/api/triggers", get(get_triggers).post(create_trigger))
        .route("/api/triggers/:id", axum::routing::put(update_trigger).delete(delete_trigger))
        .with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Dashboard API listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
