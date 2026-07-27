//! # 模型路由策略
//!
//! 根据不同类型的推理任务（如深度逻辑推理、代码生成、通用平衡对话、极速轻量任务、多模态视觉分析、结构化 JSON 输出等）
//! 统一解析和路由到最佳主模型及备用 Failover 模型，支持长文本溢出分流与超时限制规则。
//! **无静默保底方案**：若未匹配到有效的路由规则，系统将显示抛出 `ModelRouterError` 异常。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// 推理任务类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InferenceTaskType {
    /// 深度逻辑推理与分析 (Tier-1-Logic)
    Logic,
    /// 代码生成与重构 (Code-Generation)
    CodeGeneration,
    /// 通用平衡对话 (Tier-2-Balanced)
    Balanced,
    /// 极速低延迟响应 (Tier-3-Fast)
    Fast,
    /// 多模态视觉处理 (Multimodal-Vision)
    Vision,
    /// 结构化 JSON/数据提取 (Structured-Output)
    StructuredOutput,
    /// 自定义能力名称
    Custom(String),
}

impl InferenceTaskType {
    /// 获取对应的路由表配置 Key 标识符
    pub fn config_key(&self) -> &str {
        match self {
            InferenceTaskType::Logic => "Tier-1-Logic",
            InferenceTaskType::CodeGeneration => "Code-Generation",
            InferenceTaskType::Balanced => "Tier-2-Balanced",
            InferenceTaskType::Fast => "Tier-3-Fast",
            InferenceTaskType::Vision => "Multimodal-Vision",
            InferenceTaskType::StructuredOutput => "Structured-Output",
            InferenceTaskType::Custom(key) => key.as_str(),
        }
    }

    /// 从字符串解析推理任务类型
    pub fn from_key(key: &str) -> Self {
        match key {
            "Tier-1-Logic" | "Logic" | "deep-think" => InferenceTaskType::Logic,
            "Code-Generation" | "Code" | "code-gen" => InferenceTaskType::CodeGeneration,
            "Tier-2-Balanced" | "Balanced" | "general" => InferenceTaskType::Balanced,
            "Tier-3-Fast" | "Fast" | "fastest" => InferenceTaskType::Fast,
            "Multimodal-Vision" | "Vision" | "multimodal" => InferenceTaskType::Vision,
            "Structured-Output" | "Structured" | "json" => InferenceTaskType::StructuredOutput,
            other => InferenceTaskType::Custom(other.to_string()),
        }
    }
}

/// 保持向下兼容的基础设施枚举
pub type RoutingStrategy = InferenceTaskType;

/// 算力 Profile 高级路由规则
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutingRules {
    pub context_overflow_model: Option<String>,
    pub max_token_threshold: Option<usize>,
    pub timeout_ms: Option<u64>,
}

/// 算力 Profile 配置结构，关联多个任务类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityProfile {
    pub name: String,
    pub description: Option<String>,
    pub task_types: Vec<String>,
    pub primary: String,
    pub failover: Vec<String>,
    pub routing_rules: Option<RoutingRules>,
}

/// 路由错误枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelRouterError {
    /// 未找到匹配路由
    NoMatchingRoute { task_type: String },
    /// Profile 配置无效
    InvalidProfile { profile_key: String, reason: String },
}

impl fmt::Display for ModelRouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelRouterError::NoMatchingRoute { task_type } => {
                write!(f, "未找到任务类型 '{task_type}' 对应的算力 Profile 路由配置")
            }
            ModelRouterError::InvalidProfile { profile_key, reason } => {
                write!(f, "算力 Profile '{profile_key}' 配置无效: {reason}")
            }
        }
    }
}

impl std::error::Error for ModelRouterError {}

/// 路由解析结果，包含选中的主模型与容灾备选模型列表，以及溢出重定向标识
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteResult {
    pub profile_key: String,
    pub primary: String,
    pub failover: Vec<String>,
    pub routing_rules: Option<RoutingRules>,
    pub is_context_overflow: bool,
}

pub struct ModelRouter {
    pub local_model_token_limit: usize,
}

impl ModelRouter {
    pub fn new() -> Self {
        Self {
            local_model_token_limit: 4096,
        }
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRouter {
    /// 根据推理任务类型、预估 Token 数与能力路由表解析最佳模型配置。
    /// **不使用任何静默兜底方案**：若未匹配到路由或配置无效，明确返回 `ModelRouterError` 异常。
    pub fn route_task(
        &self,
        task: &InferenceTaskType,
        estimated_tokens: Option<usize>,
        config: Option<&Value>,
    ) -> Result<RouteResult, ModelRouterError> {
        let target_key = task.config_key();

        // 1. 优先在传入的 config 中匹配关联该 task_type 的 Profile
        if let Some(map) = config {
            if let Some(res) = Self::parse_and_match_profile(map, target_key, estimated_tokens)? {
                return Ok(res);
            }
        }

        // 2. 回退到默认 Profile 路由配置表
        let default_config = &*crate::core::constants::DEFAULT_CAPABILITY_ROUTING;
        if let Some(res) = Self::parse_and_match_profile(default_config, target_key, estimated_tokens)? {
            return Ok(res);
        }

        // 3. 无静默保底方案，显式抛出异常
        Err(ModelRouterError::NoMatchingRoute {
            task_type: target_key.to_string(),
        })
    }

    /// 从 JSON Map 中查找包含 target_key 的 Profile 并进行规则评估
    fn parse_and_match_profile(
        map: &Value,
        target_key: &str,
        estimated_tokens: Option<usize>,
    ) -> Result<Option<RouteResult>, ModelRouterError> {
        let obj = match map.as_object() {
            Some(o) => o,
            None => return Ok(None),
        };

        // 遍历所有 Profile entry
        for (profile_key, val) in obj {
            let profile_obj = match val.as_object() {
                Some(o) => o,
                None => continue,
            };

            // 匹配条件 1：task_types 数组中包含 target_key
            let task_types_matched = profile_obj
                .get("task_types")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().any(|item| item.as_str() == Some(target_key)))
                .unwrap_or(false);

            // 匹配条件 2：profile_key 本身匹配 target_key (兼容旧格式或无 task_types 属性的情况)
            let key_matched = profile_key == target_key;

            if task_types_matched || key_matched {
                let primary = profile_obj
                    .get("primary")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| ModelRouterError::InvalidProfile {
                        profile_key: profile_key.clone(),
                        reason: "主模型(primary)未配置或无效".to_string(),
                    })?;

                if primary.is_empty() {
                    return Err(ModelRouterError::InvalidProfile {
                        profile_key: profile_key.clone(),
                        reason: "主模型(primary)配置字符串为空".to_string(),
                    });
                }

                let failover = profile_obj
                    .get("failover")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|item| item.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                // 解析高级路由规则
                let routing_rules: Option<RoutingRules> = profile_obj
                    .get("routing_rules")
                    .and_then(|rules_val| serde_json::from_value(rules_val.clone()).ok());

                let mut final_primary = primary;
                let mut is_context_overflow = false;

                // 评估 Context Token 溢出分流规则
                if let (Some(tokens), Some(rules)) = (estimated_tokens, &routing_rules) {
                    if let (Some(threshold), Some(overflow_model)) =
                        (rules.max_token_threshold, &rules.context_overflow_model)
                    {
                        if tokens > threshold && !overflow_model.is_empty() {
                            final_primary = overflow_model.clone();
                            is_context_overflow = true;
                        }
                    }
                }

                return Ok(Some(RouteResult {
                    profile_key: profile_key.clone(),
                    primary: final_primary,
                    failover,
                    routing_rules,
                    is_context_overflow,
                }));
            }
        }

        Ok(None)
    }

    /// 简化的主模型选择函数，若匹配失败显式抛出异常 Err(ModelRouterError)
    pub fn select_best_model(
        &self,
        task: &InferenceTaskType,
        estimated_tokens: Option<usize>,
        config: Option<&Value>,
    ) -> Result<String, ModelRouterError> {
        Ok(self.route_task(task, estimated_tokens, config)?.primary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_task_type_from_and_to_key() {
        assert_eq!(InferenceTaskType::from_key("Code-Generation"), InferenceTaskType::CodeGeneration);
        assert_eq!(InferenceTaskType::CodeGeneration.config_key(), "Code-Generation");
        
        assert_eq!(InferenceTaskType::from_key("Logic"), InferenceTaskType::Logic);
        assert_eq!(InferenceTaskType::Logic.config_key(), "Tier-1-Logic");
    }

    #[test]
    fn test_default_profile_route_resolution() {
        let router = ModelRouter::new();
        
        let code_route = router.route_task(&InferenceTaskType::CodeGeneration, None, None).unwrap();
        assert_eq!(code_route.profile_key, "High-Reasoning-Profile");
        assert_eq!(code_route.primary, "claude-3-5-sonnet");
        assert!(code_route.failover.contains(&"gemini-2.5-pro".to_string()));

        let json_route = router.route_task(&InferenceTaskType::StructuredOutput, None, None).unwrap();
        assert_eq!(json_route.profile_key, "General-Balanced-Profile");
        assert_eq!(json_route.primary, "gemini-2.5-flash");
    }

    #[test]
    fn test_no_fallback_raises_error() {
        let router = ModelRouter::new();
        let empty_config = json!({});

        // 在空配置中查找非默认定义的未知任务类型，验证不使用静默兜底，直接抛出 NoMatchingRoute 错误
        let result = router.route_task(
            &InferenceTaskType::Custom("NonExistentTask".to_string()),
            None,
            Some(&empty_config),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ModelRouterError::NoMatchingRoute { task_type } => {
                assert_eq!(task_type, "NonExistentTask");
            }
            other => panic!("Expected NoMatchingRoute, got {:?}", other),
        }
    }

    #[test]
    fn test_context_overflow_model_switch() {
        let router = ModelRouter::new();
        
        let route_normal = router.route_task(&InferenceTaskType::CodeGeneration, Some(10000), None).unwrap();
        assert_eq!(route_normal.primary, "claude-3-5-sonnet");
        assert!(!route_normal.is_context_overflow);

        let route_overflow = router.route_task(&InferenceTaskType::CodeGeneration, Some(40000), None).unwrap();
        assert_eq!(route_overflow.primary, "gemini-1.5-pro");
        assert!(route_overflow.is_context_overflow);
    }
}
