use crate::gemini::types::{GenerateRequest, GenerateResponse};
use crate::utils::errors::AiNexusError;
use reqwest::{Client, StatusCode};
use std::time::Duration;

pub struct GeminiClient {
    api_key: String,
    base_url: String,
    http_client: Client,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
            http_client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn set_base_url(&mut self, url: String) {
        self.base_url = url;
    }

    /// 执行带有 Function Calling 的生成请求
    pub async fn generate_content(
        &self,
        model: &str,
        request: &GenerateRequest,
    ) -> Result<GenerateResponse, AiNexusError> {
        let url = format!("{}/{}:generateContent?key={}", self.base_url, model, self.api_key);
        
        // Simple backoff: up to 3 retries
        let mut retries = 3;
        let mut delay = Duration::from_secs(1);

        loop {
            let res = self.http_client
                .post(&url)
                .json(request)
                .send()
                .await
                .map_err(|e| AiNexusError::NetworkError(e.to_string()))?;

            let status = res.status();
            if status.is_success() {
                let parsed: GenerateResponse = res.json().await
                    .map_err(|e| AiNexusError::ParseError(e.to_string()))?;
                return Ok(parsed);
            } else if (status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()) && retries > 0 {
                retries -= 1;
                tokio::time::sleep(delay).await;
                delay *= 2;
                continue;
            }
            
            let err_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiNexusError::GeminiApiError(format!("HTTP {}: {}", status, err_text)));
        }
    }
}
