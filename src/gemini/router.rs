//! # 模型路由策略
//!
//! `RoutingStrategy` 是 Gemini 提供商层的内部路由枚举，
//! 属于基础设施细节，**有意独立于** `core::interfaces` 契约，
//! 避免上层业务对 LLM 提供商路由机制产生直接依赖。
pub enum RoutingStrategy {
    Fastest,    // 侧重低延迟 (Flash Lite / 8b)
    Balanced,   // 默认策略 (Flash)
    DeepThink,  // 侧重逻辑能力 (Pro)
}

pub struct ModelRouter {
    pub local_model_token_limit: usize,
}

impl ModelRouter {
    pub fn new() -> Self {
        Self {
            local_model_token_limit: 4096, // Just a default
        }
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRouter {
    pub fn select_best_model(&self, strategy: RoutingStrategy, _estimated_tokens: usize) -> String {
        let config = crate::core::config::get_config();
        match strategy {
            RoutingStrategy::Fastest => "gemini-2.5-flash-lite".to_string(),
            RoutingStrategy::Balanced => config.models.global_gemini_model.clone(),
            RoutingStrategy::DeepThink => "gemini-1.5-pro".to_string(),
        }
    }
}
