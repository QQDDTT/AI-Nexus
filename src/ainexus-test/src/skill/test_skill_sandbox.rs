use ai_nexus::skill::sandbox::WasmSandbox;
use serde_json::json;

#[tokio::test]
async fn test_wasm_sandbox_infinite_loop_trap() {
    let sandbox = WasmSandbox::new().expect("Failed to init sandbox");

    let wat = r#"
        (module
            (memory (export "memory") 1)
            (func $alloc (export "alloc") (param i32) (result i32) (i32.const 0))
            (func $dealloc (export "dealloc") (param i32 i32))
            (func $loop_func (export "execute") (param i32 i32) (result i64)
                loop $my_loop
                    br $my_loop
                end
                i64.const 0
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("Failed to parse WAT");

    let params = json!({"test": 123});
    // 给予很少的燃料
    let result = sandbox.execute_wasm(&wasm_bytes, params, 100).await;
    
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("out of fuel"), "Expected out of fuel trap, got: {}", err_str);
}

#[tokio::test]
async fn test_wasm_sandbox_json_ipc() {
    // 1. 编译 test-wasm-skill 项目为 wasm32-unknown-unknown
    let mut cmd = std::process::Command::new("cargo");
    for (key, _) in std::env::vars() {
        if key.starts_with("CARGO_") {
            cmd.env_remove(key);
        }
    }

    let status = cmd
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .current_dir("test-wasm-skill")
        .status()
        .expect("Failed to run cargo build for test-wasm-skill");

    assert!(status.success(), "Failed to compile WASM skill");

    // 2. 加载生成的 .wasm 文件 (注意 target 目录通常在 workspace 根目录, 也就是 ../../target)
    let wasm_path = "../../target/wasm32-unknown-unknown/release/test_wasm_skill.wasm";
    let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read wasm file");

    // 3. 跑沙箱
    let sandbox = WasmSandbox::new().expect("Failed to init sandbox");
    let input = json!({
        "command": "say_hello",
        "user_id": 999
    });

    // 给予足够的燃料，比如 50_000_000 (反序列化比较费燃料)
    let output = sandbox.execute_wasm(&wasm_bytes, input.clone(), 50_000_000).await.expect("Execution failed");

    // 4. 验证返回值
    assert_eq!(output["success"].as_bool(), Some(true));
    assert_eq!(output["received"], input);
    assert_eq!(output["message"].as_str(), Some("Hello from Wasm Sandbox!"));
}
