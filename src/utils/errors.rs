use thiserror::Error;

/// 贯穿全系统的核心业务错误集
#[derive(Error, Debug)]
pub enum AiNexusError {
    #[error("Model API quota exceeded, downgrading route")]
    ApiQuotaExceeded,
    
    #[error("WASM Sandbox execution failed for '{skill_name}': {reason}")]
    SkillExecutionFailed {
        skill_name: String,
        reason: String,
    },
    
    #[error("Agent context corrupted or memory lost")]
    AgentContextCorrupted,
    
    #[error("General error: {0}")]
    General(String),
    
    #[error("Network request failed: {0}")]
    NetworkError(String),
    
    #[error("Failed to parse API response: {0}")]
    ParseError(String),
    
    #[error("Gemini API error: {0}")]
    GeminiApiError(String),
}
