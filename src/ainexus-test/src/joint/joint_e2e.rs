use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

const BASE_URL: &str = "http://localhost:3000";

fn create_test_app_state(dir_path: &std::path::Path) -> ai_nexus::os::api::AppState {
    let storage = std::sync::Arc::new(ai_nexus::storage::Storage::new(dir_path.to_str().unwrap()).unwrap());
    let gemini_client = std::sync::Arc::new(ai_nexus::gemini::client::GeminiClient::new("fake_key".to_string()));
    let embedding_client = std::sync::Arc::new(ai_nexus::gemini::embedding::GeminiEmbeddingClient::new("fake_key".to_string()));
    let skill_registry = std::sync::Arc::new(ai_nexus::skill::registry::GraphSkillRegistry::new(
        storage.graph_store.clone(),
        storage.vector_store.clone(),
        embedding_client,
    ));
    ai_nexus::os::api::AppState {
        storage,
        skill_registry,
        gemini_client,
    }
}

#[tokio::test]
async fn run_full_e2e_suite() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app_state = create_test_app_state(temp_dir.path());
    
    // 启动后台服务器
    tokio::spawn(async move {
        ai_nexus::os::api::start_server(app_state).await;
    });
    // 等待服务器启动
    sleep(Duration::from_millis(100)).await;

    let client = Client::new();

    // 1. Negative Auth (Unauthorized Access)
    let res = client.get(format!("{}/api/dashboard/stats", BASE_URL)).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "Unauthenticated access should be blocked with 401");

    // 2. Auth Login (Wrong password)
    let res = client.post(format!("{}/api/auth/login", BASE_URL))
        .json(&json!({"username": "admin", "password": "wrong_password"}))
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "Wrong password should be blocked with 401");

    // 3. Auth Login (Success)
    let res = client.post(format!("{}/api/auth/login", BASE_URL))
        .json(&json!({"username": "admin", "password": "admin123"}))
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "Login should succeed with 200");
    
    let body: Value = res.json().await.unwrap();
    let token = body["token"].as_str().expect("Token should be present").to_string();
    assert!(!token.is_empty(), "Token should not be empty");

    let auth_header = format!("Bearer {}", token);

    // 4. Dashboard Stats & Resilience (Graceful Degradation)
    let res = client.get(format!("{}/api/dashboard/stats", BASE_URL))
        .header("Authorization", &auth_header)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "Dashboard stats should return 200");
    
    let stats: Value = res.json().await.unwrap();
    assert!(
        stats["api_health_trend"].as_str().unwrap().contains("NexusDB"), 
        "API health trend should reflect NexusDB status"
    );
    assert!(
        stats["gateways"].as_array().unwrap().len() >= 1,
        "Gateways array should have at least 1 gateway"
    );

    let res = client.get(format!("{}/api/dashboard/token-trend", BASE_URL))
        .header("Authorization", &auth_header)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 5. Gateways (Read)
    let res = client.get(format!("{}/api/gateways", BASE_URL))
        .header("Authorization", &auth_header)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.json::<Value>().await.unwrap().is_array());

    // Gateway (Toggle non-existent ID -> 404)
    let res = client.post(format!("{}/api/gateways/test-id/toggle", BASE_URL))
        .header("Authorization", &auth_header)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND, "Toggling non-existent gateway ID should return 404");

    // 6. Settings (Read & Update)
    let res = client.get(format!("{}/api/settings", BASE_URL))
        .header("Authorization", &auth_header)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let mut settings = res.json::<Value>().await.unwrap();
    assert!(!settings["db_path"].as_str().unwrap().is_empty());

    settings["log_masking"] = json!(true);
    let res = client.put(format!("{}/api/settings", BASE_URL))
        .header("Authorization", &auth_header)
        .json(&settings)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Negative setting (Bad JSON format test)
    let res = client.put(format!("{}/api/settings", BASE_URL))
        .header("Authorization", &auth_header)
        .header("Content-Type", "application/json")
        .body("{ bad_json: ")
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST, "Malformed JSON should return 400 Bad Request");

    // 7. Sessions Negative (Invalid delete id format or not found)
    let res = client.delete(format!("{}/api/sessions/{}", BASE_URL, "invalid_random_id"))
        .header("Authorization", &auth_header)
        .send().await.unwrap();
    assert_ne!(res.status(), StatusCode::INTERNAL_SERVER_ERROR, "Invalid session delete should not panic (no 500)");

    // 8. Ledger (Read)
    let res = client.get(format!("{}/api/ledger", BASE_URL))
        .header("Authorization", &auth_header)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_model_router_e2e_profile_resolutions() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app_state = create_test_app_state(temp_dir.path());
    
    tokio::spawn(async move {
        ai_nexus::os::api::start_server(app_state).await;
    });
    sleep(Duration::from_millis(100)).await;

    let client = Client::new();

    // 登录获取 Token
    let res = client.post(format!("{}/api/auth/login", BASE_URL))
        .json(&json!({"username": "admin", "password": "admin123"}))
        .send().await.unwrap();
    let body: Value = res.json().await.unwrap();
    let token = body["token"].as_str().unwrap().to_string();
    let auth_header = format!("Bearer {}", token);

    // E2E-LINK-03: 验证正常 Profile 路由匹配 (Code-Generation)
    let res = client.get(format!("{}/api/models/routing/resolve?task_type=Code-Generation", BASE_URL))
        .header("Authorization", &auth_header)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "Model router resolve should return 200 OK");
    let route_json: Value = res.json().await.unwrap();
    assert_eq!(route_json["profile_key"].as_str().unwrap(), "High-Reasoning-Profile");
    assert_eq!(route_json["primary"].as_str().unwrap(), "claude-3-5-sonnet");

    // E2E-LINK-04: 验证 Context Token 溢出分流 (estimated_tokens = 40000 > 32768)
    let res = client.get(format!("{}/api/models/routing/resolve?task_type=Code-Generation&estimated_tokens=40000", BASE_URL))
        .header("Authorization", &auth_header)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let overflow_json: Value = res.json().await.unwrap();
    assert_eq!(overflow_json["primary"].as_str().unwrap(), "gemini-1.5-pro");
    assert_eq!(overflow_json["is_context_overflow"].as_bool().unwrap(), true);

    // E2E-LINK-05: 验证无静默保底抛出 400 (NoMatchingRoute)
    let res = client.get(format!("{}/api/models/routing/resolve?task_type=UnknownNonExistentTask", BASE_URL))
        .header("Authorization", &auth_header)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST, "Unknown task should fail with 400 Bad Request, no silent fallback");
    let err_json: Value = res.json().await.unwrap();
    assert_eq!(err_json["error"].as_str().unwrap(), "NoMatchingRoute");
}

#[tokio::test]
async fn test_acp_trace_id_lifecycle() {
    use ai_nexus::core::{AcpMessage, AcpPayload, Component};
    use ai_nexus::os::bus::NexusBus;
    use uuid::Uuid;

    let mut bus = NexusBus::new(1024);
    let sender = bus.get_sender();
    let mut receiver = bus.take_receiver().unwrap();

    let target_trace_id = Uuid::new_v4().to_string();
    
    let req_msg = AcpMessage {
        trace_id: target_trace_id.clone(),
        source: Component::NexusOS,
        target: Component::ModelRouter,
        timestamp: 0,
        payload: AcpPayload::InferenceRequest {
            prompt: "Ping test".to_string(),
            target_model: "gemini-2.5-flash".to_string(),
        },
    };

    sender.send(req_msg).await.expect("Failed to send initial ACP message");

    if let Some(routed_msg) = receiver.recv().await {
        assert_eq!(routed_msg.trace_id, target_trace_id, "TraceID must propagate through the NexusBus routing");
        assert_eq!(routed_msg.target, Component::ModelRouter, "Target must be correctly routed");
    } else {
        panic!("Bus dropped the message");
    }
}
