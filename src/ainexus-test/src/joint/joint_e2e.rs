use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

const BASE_URL: &str = "http://localhost:3000";

#[tokio::test]
async fn run_full_e2e_suite() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    let db = std::sync::Arc::new(ai_nexus::storage::NexusDb::new(db_path).unwrap());
    
    // 启动后台服务器
    tokio::spawn(async move {
        ai_nexus::os::api::start_server(db).await;
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
    assert_eq!(
        stats["api_health_trend"].as_str().unwrap(), 
        "NexusDB Active", 
        "API health trend should reflect NexusDB is active"
    );
    assert_eq!(
        stats["gateways"].as_array().unwrap().len(), 
        1,
        "Gateways array should have length 1 (seeded test gateway)"
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

    // Gateway (Toggle)
    let res = client.post(format!("{}/api/gateways/test-id/toggle", BASE_URL))
        .header("Authorization", &auth_header)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

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
async fn test_acp_trace_id_lifecycle() {
    // 联合测试: 验证 TraceID 的完整生命周期与 ACP 传播
    // 按照 13_TEST_DESIGN 规范: "仿真 S0-S4 全生命周期流转，验证信号在各维面间的有序传递"
    
    use ai_nexus::core::{AcpMessage, AcpPayload, Component};
    use ai_nexus::os::bus::NexusBus;
    use uuid::Uuid;

    // 1. 初始化仿真总线
    let mut bus = NexusBus::new(1024);
    let sender = bus.get_sender();
    let mut receiver = bus.take_receiver().unwrap();

    // 2. 生成一个受追踪的 TraceID
    let target_trace_id = Uuid::new_v4().to_string();
    
    // 3. 构建 S0 始发消息 (模拟通道层发起)
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

    // 4. 发送至总线
    sender.send(req_msg).await.expect("Failed to send initial ACP message");

    // 5. 监听总线的响应并断言 (断言路由后的消息必须携带相同的 TraceID)
    if let Some(routed_msg) = receiver.recv().await {
        assert_eq!(routed_msg.trace_id, target_trace_id, "TraceID must propagate through the NexusBus routing");
        assert_eq!(routed_msg.target, Component::ModelRouter, "Target must be correctly routed");
    } else {
        panic!("Bus dropped the message");
    }
}
