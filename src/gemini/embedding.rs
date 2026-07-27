use crate::core::interfaces::EmbeddingProvider;
use crate::utils::errors::AiNexusError;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct EmbedRequest {
    model: String,
    content: EmbedContent,
}

#[derive(Debug, Serialize)]
struct EmbedContent {
    parts: Vec<EmbedPart>,
}

#[derive(Debug, Serialize)]
struct EmbedPart {
    text: String,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embedding: EmbeddingValues,
}

#[derive(Debug, Deserialize)]
struct EmbeddingValues {
    values: Vec<f32>,
}

pub struct GeminiEmbeddingClient {
    api_key: String,
    base_url: String,
    http_client: Client,
}

impl GeminiEmbeddingClient {
    pub fn new(api_key: String) -> Self {
        Self::new_with_url(api_key, None)
    }

    pub fn new_with_url(api_key: String, base_url: Option<String>) -> Self {
        let config = crate::core::config::get_config();
        let default_url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent", config.models.embedding_model);
        let target_url = base_url.filter(|u| !u.trim().is_empty()).unwrap_or(default_url);
        Self {
            api_key,
            base_url: target_url,
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for GeminiEmbeddingClient {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, AiNexusError> {
        let config = crate::core::config::get_config();
        let req = EmbedRequest {
            model: format!("models/{}", config.models.embedding_model),
            content: EmbedContent {
                parts: vec![EmbedPart {
                    text: text.to_string(),
                }],
            },
        };

        let url = format!("{}?key={}", self.base_url, self.api_key);

        let res = self
            .http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await
            .map_err(|e| AiNexusError::NetworkError(e.to_string()))?;

        let status = res.status();
        if status.is_success() {
            let parsed: EmbedResponse = res
                .json()
                .await
                .map_err(|e| AiNexusError::ParseError(e.to_string()))?;
            Ok(parsed.embedding.values)
        } else {
            let err_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(AiNexusError::GeminiApiError(format!("HTTP {}: {}", status, err_text)))
        }
    }
}
