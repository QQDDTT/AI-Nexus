use ai_nexus::gemini::client::GeminiClient;
use ai_nexus::gemini::types::{GenerateRequest, Content, Part};
use mockito::Server;

#[tokio::test]
async fn test_generate_content_success() {
    let mut server = Server::new_async().await;
    
    // 模拟 Google API 的成功响应
    let mock = server.mock("POST", "/gemini-2.5-flash:generateContent?key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "candidates": [
                {
                    "content": {
                        "role": "model",
                        "parts": [
                            { "text": "Hello from mock Gemini!" }
                        ]
                    },
                    "finish_reason": "STOP"
                }
            ]
        }"#)
        .create_async().await;

    // 为了允许注入测试 URL，我们需要给 GeminiClient 添加个改变 base_url 的能力
    // 或者我们在测试里强制改用一个假的 url
    let mut client = GeminiClient::new("fake_api_key".to_string());
    
    // 因为 GeminiClient 的 base_url 是私有的，我们需要一个专门用于测试的方法来覆盖它
    // 但作为简化，我们可以用 unsafe 或者在 client.rs 里提供一个 with_base_url
    
    // 这里我们假设我们在 client.rs 里加了 set_base_url 或者是通过 mock 拦截
    // 因为这只是个示例，我们先改 client.rs 让它可以设 base_url
    client.set_base_url(server.url());

    let req = GenerateRequest {
        contents: vec![Content {
            role: "user".to_string(),
            parts: vec![Part {
                text: Some("Hello".to_string()),
                function_call: None,
                function_response: None,
            }],
        }],
        system_instruction: None,
        tools: None,
    };

    let result = client.generate_content("gemini-2.5-flash", &req).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.candidates.is_some());
    let candidates = response.candidates.unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].content.parts[0].text.as_ref().unwrap(), "Hello from mock Gemini!");
    
    mock.assert_async().await;
}
