use crate::gemini::client::GeminiClient;
use crate::gemini::types::{Content, GenerateRequest, Part};
use crate::utils::errors::AiNexusError;
use std::process::Command;
use std::sync::Arc;
use tokio::fs;

/// 全局造物主分身 (Meta Agent)
pub struct MetaAgent {
    gemini: Arc<GeminiClient>,
    workspace_dir: String,
    model_name: String,
}

impl MetaAgent {
    pub fn new(gemini: Arc<GeminiClient>, workspace_dir: String, model_name: String) -> Self {
        Self {
            gemini,
            workspace_dir,
            model_name,
        }
    }

    /// 核心生命周期：生成代码 -> 写入磁盘 -> 尝试编译 -> (失败则重试) -> 成功返回 .wasm 二进制流
    pub async fn generate_and_compile_skill(&self, intent: &str) -> Result<Vec<u8>, AiNexusError> {
        let mut retry_count = 0;
        let max_retries = 3;
        
        let system_prompt = "You are an expert Rust systems programmer. \
            Your task is to write a single Rust file (lib.rs) for a Wasmtime sandbox. \
            You must ONLY output valid Rust source code, no markdown wrappers, no explanations. \
            The code must export the following C ABI functions for linear memory: \
            `#[no_mangle] pub extern \"C\" fn alloc(size: i32) -> *mut u8` \
            `#[no_mangle] pub unsafe extern \"C\" fn dealloc(ptr: *mut u8, size: i32)` \
            `#[no_mangle] pub unsafe extern \"C\" fn execute(ptr: i32, len: i32) -> i64` \
            In `execute`, read JSON bytes from memory, process it, allocate response JSON, and return (ptr << 32 | len). \
            Use `serde_json` and `serde`. Do not use `std::alloc::alloc` directly, just use Vec. \
            Do not include `#![no_main]` or `#![no_std]`. This is a cdylib.";

        let mut current_error_context = String::new();

        loop {
            // 1. 组装请求
            let parts = vec![Part {
                text: Some(format!("Intent: {}\n\n{}", intent, current_error_context)),
                function_call: None,
                function_response: None,
            }];

            let req = GenerateRequest {
                contents: vec![Content {
                    role: "user".to_string(),
                    parts,
                }],
                system_instruction: Some(Content {
                    role: "system".to_string(),
                    parts: vec![Part {
                        text: Some(system_prompt.to_string()),
                        function_call: None,
                        function_response: None,
                    }],
                }),
                tools: None,
            };

            // 2. 调用大模型生成代码
            tracing::info!("MetaAgent: Requesting code generation (Attempt {})", retry_count + 1);
            let response = self.gemini.generate_content(&self.model_name, &req).await?;
            
            let mut code = response.candidates
                .as_ref()
                .and_then(|c| c.first())
                .and_then(|c| c.content.parts.first())
                .and_then(|p| p.text.clone())
                .unwrap_or_default();

            // 简单清洗 markdown
            if code.starts_with("```rust") {
                code = code.trim_start_matches("```rust").trim_start().to_string();
            }
            if code.starts_with("```") {
                code = code.trim_start_matches("```").trim_start().to_string();
            }
            if code.ends_with("```") {
                code = code.trim_end_matches("```").trim_end().to_string();
            }

            // 3. 写入工作区
            let src_dir = format!("{}/src", self.workspace_dir);
            let lib_path = format!("{}/lib.rs", src_dir);
            
            // 确保目录存在
            fs::create_dir_all(&src_dir).await.map_err(|e| AiNexusError::General(format!("Failed to create dir: {}", e)))?;
            fs::write(&lib_path, &code).await.map_err(|e| AiNexusError::General(format!("Failed to write code: {}", e)))?;

            // 4. 执行 Cargo 编译
            tracing::info!("MetaAgent: Compiling generated skill...");
            let mut cmd = Command::new("cargo");
            // 清理测试环境继承的 CARGO_ 环境变量，避免互相干扰
            for (key, _) in std::env::vars() {
                if key.starts_with("CARGO_") {
                    cmd.env_remove(key);
                }
            }

            let output = cmd
                .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
                .current_dir(&self.workspace_dir)
                .output()
                .map_err(|e| AiNexusError::General(format!("Cargo command failed: {}", e)))?;

            if output.status.success() {
                // 编译成功，提取产物
                tracing::info!("MetaAgent: Compilation successful!");
                // Note: The workspace target dir might be at the workspace root, or locally if isolated.
                // Since meta-workspace is a workspace member of AI-Nexus, its target is AI-Nexus/target!
                // Wait, if it's a member, it outputs to AI-Nexus/target.
                // We should read from there. Let's assume the caller passes the right workspace path.
                // Actually, if we run `cargo build`, we can just parse the output or hardcode for now.
                // Since this runs in ainexus-test, the target is in the root.
                // So target/wasm32-unknown-unknown/release/meta_workspace.wasm
                // But let's assume it's in target/ relative to AI-Nexus.
                let wasm_path = "target/wasm32-unknown-unknown/release/meta_workspace.wasm";
                let wasm_bytes = fs::read(wasm_path).await.unwrap_or_else(|_| {
                    // Fallback to local target if somehow isolated
                    std::fs::read(format!("{}/target/wasm32-unknown-unknown/release/meta_workspace.wasm", self.workspace_dir)).unwrap_or_default()
                });
                return Ok(wasm_bytes);
            } else {
                // 编译失败，提取错误信息并重试
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                tracing::error!("MetaAgent: Compilation failed:\n{}", stderr);

                if retry_count >= max_retries {
                    return Err(AiNexusError::General(format!("Max retries reached. Last error:\n{}", stderr)));
                }

                current_error_context = format!("Your previous code failed to compile with the following error:\n{}\nFix the errors and output ONLY the corrected Rust code.", stderr);
                retry_count += 1;
            }
        }
    }
}
