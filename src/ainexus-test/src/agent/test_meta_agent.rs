use ai_nexus::agent::MetaAgent;
use ai_nexus::gemini::client::GeminiClient;
use mockito::Server;
use std::sync::Arc;

#[tokio::test]
async fn test_meta_agent_skill_creation() {
    // 1. 启动一个 Mock Server，充当 Gemini API
    let mut server = Server::new_async().await;

    // 第一次调用大模型，返回故意带有语法错误的 Rust 代码
    let _mock_fail = server.mock("POST", "/v1beta/models/gemini-2.5-flash:generateContent?key=fake_key")
        .expect(1)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::to_string(&serde_json::json!({
                "candidates": [{
                    "content": {
                        "role": "model",
                        "parts": [{"text": "```rust\nthis_is_invalid_rust_code_for_sure!!!\n```"}]
                    }
                }]
            })).unwrap()
        )
        .create_async()
        .await;

    // 第二次调用大模型 (因为重试)，返回合法的完整 Rust 代码
    let valid_rust = r#"
        use serde_json::Value;
        #[no_mangle]
        pub extern "C" fn alloc(size: i32) -> *mut u8 {
            let mut buf = Vec::with_capacity(size as usize);
            let ptr = buf.as_mut_ptr();
            std::mem::forget(buf);
            ptr
        }
        #[no_mangle]
        pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: i32) {
            let _ = Vec::from_raw_parts(ptr, 0, size as usize);
        }
        #[no_mangle]
        pub unsafe extern "C" fn execute(ptr: i32, len: i32) -> i64 {
            let _input = std::slice::from_raw_parts(ptr as *const u8, len as usize);
            let response = serde_json::json!({"meta": "success"});
            let out_str = serde_json::to_string(&response).unwrap().into_bytes();
            let out_len = out_str.len() as i32;
            let out_ptr = out_str.as_ptr() as i32;
            std::mem::forget(out_str);
            ((out_ptr as i64) << 32) | (out_len as i64)
        }
    "#;

    let _mock_success = server.mock("POST", "/v1beta/models/gemini-2.5-flash:generateContent?key=fake_key")
        .expect(1) // 第二次请求
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::to_string(&serde_json::json!({
                "candidates": [{
                    "content": {
                        "role": "model",
                        "parts": [{"text": valid_rust}]
                    }
                }]
            })).unwrap()
        )
        .create_async()
        .await;

    // 2. 初始化 MetaAgent
    let mut gemini = GeminiClient::new("fake_key".to_string());
    gemini.set_base_url(server.url() + "/v1beta/models");
    
    let meta_agent = MetaAgent::new(
        Arc::new(gemini),
        "meta-workspace".to_string(), // 因为测试运行在 src/ainexus-test 下
        "gemini-2.5-flash".to_string()
    );

    // 3. 执行生成
    let result = meta_agent.generate_and_compile_skill("Create a skill that returns success").await;

    // 4. 验证
    assert!(result.is_ok(), "Expected success after retry, but got error: {:?}", result.err());
    let wasm_bytes = result.unwrap();
    assert!(!wasm_bytes.is_empty());
    
    // 简单验证产出是否是 Wasm 魔数 "\0asm"
    assert_eq!(&wasm_bytes[0..4], b"\0asm");
}
