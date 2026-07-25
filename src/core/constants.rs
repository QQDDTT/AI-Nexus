use serde_json::{json, Value};
use std::sync::LazyLock;

/// 默认能力路由映射配置表
pub static DEFAULT_CAPABILITY_ROUTING: LazyLock<Value> = LazyLock::new(|| {
    json!({
        "Tier-1-Logic": {
            "primary": "gemini-2.5-pro",
            "failover": ["gpt-4o", "claude-3-5-sonnet"]
        },
        "Tier-2-Balanced": {
            "primary": "gemini-1.5-flash",
            "failover": ["llama-3-70b"]
        },
        "Tier-3-Fast": {
            "primary": "gemini-2.5-flash",
            "failover": ["llama-3-8b"]
        },
        "Multimodal-Vision": {
            "primary": "gemini-1.5-pro",
            "failover": ["gpt-4o-vision"]
        }
    })
});

/// 认证相关常量
pub const AUTH_TOKEN_PREFIX: &str = "Bearer ";
pub const DEFAULT_ADMIN_USERNAME: &str = "admin";
pub const DEFAULT_ADMIN_PASSWORD: &str = "admin123";
pub const STUB_AUTH_TOKEN: &str = "stub_token_12345";
