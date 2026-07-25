use crate::core::interfaces::Skill;
use crate::utils::errors::AiNexusError;
use async_trait::async_trait;
use serde_json::Value;
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};

/// 网页检索技能 (WebSearchSkill)
/// 允许大模型通过原生的 HTTP 客户端发起检索，默认使用 Wikipedia API。
pub struct WebSearchSkill {
    client: reqwest::Client,
}

impl WebSearchSkill {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Skill for WebSearchSkill {
    fn name(&self) -> &str {
        "web_search"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "name": "web_search",
            "description": "Makes an HTTP request to an external network API.",
            "parameters": {
                "type": "object",
                "properties": {
                    "domain": {
                        "type": "string",
                        "description": "Target domain or IP address, e.g. 'api.github.com' or '192.168.1.10'"
                    },
                    "endpoint": {
                        "type": "string",
                        "description": "Request path, e.g. '/search/repositories'"
                    },
                    "method": {
                        "type": "string",
                        "description": "HTTP method, e.g. 'GET', 'POST', 'PUT', 'DELETE'"
                    },
                    "params": {
                        "type": "object",
                        "description": "URL query parameters"
                    },
                    "headers": {
                        "type": "object",
                        "description": "HTTP request headers"
                    },
                    "body": {
                        "type": "string",
                        "description": "Request body, required for POST/PUT (JSON string or plain text)"
                    }
                },
                "required": ["domain", "endpoint", "method"]
            }
        })
    }

    async fn execute(&self, params: Value) -> Result<Value, AiNexusError> {
        let domain = params.get("domain").and_then(|v| v.as_str()).unwrap_or_default();
        let endpoint = params.get("endpoint").and_then(|v| v.as_str()).unwrap_or_default();
        let method_str = params.get("method").and_then(|v| v.as_str()).unwrap_or_default().to_uppercase();

        if domain.is_empty() || endpoint.is_empty() || method_str.is_empty() {
            return Err(AiNexusError::General("domain, endpoint, and method are required".to_string()));
        }

        let url = format!("https://{}{}", domain, endpoint);

        let method = match method_str.as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            _ => return Err(AiNexusError::General(format!("Unsupported HTTP method: {}", method_str))),
        };

        let mut request = self.client.request(method, &url);

        if let Some(query_params) = params.get("params").and_then(|v| v.as_object()) {
            request = request.query(query_params);
        }

        if let Some(headers) = params.get("headers").and_then(|v| v.as_object()) {
            for (key, value) in headers {
                if let Some(val_str) = value.as_str() {
                    request = request.header(key, val_str);
                }
            }
        }

        if let Some(body) = params.get("body") {
            if let Some(body_str) = body.as_str() {
                request = request.body(body_str.to_string());
            } else {
                request = request.json(body);
            }
        }

        let response = match request.send().await {
            Ok(resp) => resp,
            Err(e) => {
                return Ok(serde_json::json!({
                    "error": format!("Network request failed: {}", e)
                }));
            }
        };

        let status_code = response.status().as_u16();
        let mut resp_headers = serde_json::Map::new();
        for (k, v) in response.headers() {
            if let Ok(val_str) = v.to_str() {
                resp_headers.insert(k.as_str().to_string(), serde_json::Value::String(val_str.to_string()));
            }
        }

        let text_result = response.text().await.unwrap_or_default();
        let body_val: Value = serde_json::from_str(&text_result).unwrap_or(serde_json::Value::String(text_result));

        Ok(serde_json::json!({
            "status_code": status_code,
            "headers": resp_headers,
            "body": body_val
        }))
    }
}

/// 文件生成技能 (FileGenerateSkill)
/// 允许落盘生成文件，强隔离在 ./data 目录下。
pub struct FileGenerateSkill;

#[async_trait]
impl Skill for FileGenerateSkill {
    fn name(&self) -> &str {
        "file_generate"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "name": "file_generate",
            "description": "Generates a file with specified content. The path is strictly relative to the ./data/ directory.",
            "parameters": {
                "type": "object",
                "properties": {
                    "filename": {
                        "type": "string",
                        "description": "The name or relative path of the file (e.g. 'output.json')."
                    },
                    "content": {
                        "type": "string",
                        "description": "The raw text content to write into the file."
                    },
                    "format": {
                        "type": "string",
                        "description": "File format identifier, e.g. 'json', 'markdown'."
                    },
                    "encoding": {
                        "type": "string",
                        "description": "File encoding format, default 'utf-8'."
                    }
                },
                "required": ["filename", "content"]
            }
        })
    }

    fn requires_human_approval(&self) -> bool {
        // 敏感操作，建议审批
        true
    }

    async fn execute(&self, params: Value) -> Result<Value, AiNexusError> {
        let filename = params.get("filename").and_then(|v| v.as_str()).unwrap_or_default();
        let content = params.get("content").and_then(|v| v.as_str()).unwrap_or_default();
        let _format = params.get("format").and_then(|v| v.as_str()).unwrap_or("plaintext");
        let encoding = params.get("encoding").and_then(|v| v.as_str()).unwrap_or("utf-8");

        if filename.is_empty() {
            return Err(AiNexusError::General("Filename cannot be empty".to_string()));
        }

        if encoding.to_lowercase() != "utf-8" {
            return Err(AiNexusError::General(format!("Unsupported encoding: {}", encoding)));
        }

        // 安全校验：防止跨目录攻击 (e.g., ../../etc/passwd)
        let normalized_path = Path::new(filename);
        if normalized_path.is_absolute() || normalized_path.components().any(|c| c == std::path::Component::ParentDir) {
            return Err(AiNexusError::General("Access denied: Paths must be relative and cannot contain '..'".to_string()));
        }

        let config = crate::core::config::get_config();
        let base_dir = PathBuf::from(&config.system.storage_path).join("generated");
        tokio::fs::create_dir_all(&base_dir).await.map_err(|e| {
            AiNexusError::General(format!("Failed to create base directory: {}", e))
        })?;

        let target_path = base_dir.join(normalized_path);
        let content_bytes = content.as_bytes();

        tokio::fs::write(&target_path, content_bytes).await.map_err(|e| {
            AiNexusError::General(format!("Failed to write file: {}", e))
        })?;

        let mut hasher = Sha256::new();
        hasher.update(content_bytes);
        let result = hasher.finalize();
        let checksum = format!("{:x}", result);

        Ok(serde_json::json!({
            "success": true,
            "file_path": target_path.display().to_string(),
            "size_bytes": content_bytes.len(),
            "checksum": checksum,
            "audit_status": "Passed"
        }))
    }
}
