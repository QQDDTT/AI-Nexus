use serde_json::{json, Value};
use std::sync::LazyLock;

/// 默认能力路由映射配置表
pub static DEFAULT_CAPABILITY_ROUTING: LazyLock<Value> = LazyLock::new(|| {
    json!({
        "High-Reasoning-Profile": {
            "name": "深度智力与代码算力组",
            "description": "适用于高逻辑、复杂代码分析与多步骤推理任务",
            "task_types": ["Tier-1-Logic", "Code-Generation"],
            "primary": "claude-3-5-sonnet",
            "failover": ["gemini-2.5-pro", "gpt-4o"],
            "routing_rules": {
                "context_overflow_model": "gemini-1.5-pro",
                "max_token_threshold": 32768,
                "timeout_ms": 10000
            }
        },
        "General-Balanced-Profile": {
            "name": "通用对话与极速响应组",
            "description": "适用于日常对话、低延迟回复与轻量结构化提取",
            "task_types": ["Tier-2-Balanced", "Tier-3-Fast", "Structured-Output"],
            "primary": "gemini-2.5-flash",
            "failover": ["llama-3-8b"],
            "routing_rules": {
                "context_overflow_model": "gemini-1.5-flash",
                "max_token_threshold": 16384,
                "timeout_ms": 5000
            }
        },
        "Multimodal-Vision-Profile": {
            "name": "多模态视觉处理组",
            "description": "视觉分析、图像理解与多模态生成",
            "task_types": ["Multimodal-Vision"],
            "primary": "gemini-1.5-pro",
            "failover": ["gpt-4o-vision"],
            "routing_rules": {}
        }
    })
});

/// 认证相关常量
pub const AUTH_TOKEN_PREFIX: &str = "Bearer ";
pub const DEFAULT_ADMIN_USERNAME: &str = "admin";
pub const DEFAULT_ADMIN_PASSWORD: &str = "admin123";
pub const STUB_AUTH_TOKEN: &str = "stub_token_12345";
