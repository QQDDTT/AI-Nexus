use crate::storage::NexusDb;
use serde_json::json;
use std::sync::Arc;
use thiserror::Error;

/// IAM 模块独立错误类型。
///
/// **有意独立于** `utils::errors::AiNexusError`，
/// 防止配额/鉴权细节泄漏至核心业务层。
/// 调用方（`main.rs`）在捕获后自行决定是否向上转换。
#[derive(Error, Debug)]
pub enum IamError {
    #[error("Quota exceeded: required {required}, but only have {balance} tokens remaining")]
    QuotaExceeded { required: u32, balance: u32 },
    #[error("Database error: {0}")]
    DatabaseError(#[from] anyhow::Error),
}

/// 身份与访问管理网关
pub struct IamGateway {
    nexus_db: Arc<NexusDb>,
}

impl IamGateway {
    pub fn new(nexus_db: Arc<NexusDb>) -> Self {
        Self { nexus_db }
    }

    /// 校验并在请求前预扣减额度
    pub async fn verify_and_deduct_quota(&self, user_id: &str, estimated_tokens: u32) -> Result<(), IamError> {
        let collection = self.nexus_db.collection("users");
        
        let mut balance = 5000; // default for new users

        if let Some(user_data) = collection.get(user_id) {
            if let Some(b) = user_data.get("balance").and_then(|v| v.as_u64()) {
                balance = b as u32;
            }
        }

        if balance < estimated_tokens {
            return Err(IamError::QuotaExceeded {
                required: estimated_tokens,
                balance,
            });
        }

        // 扣减额度
        let new_balance = balance - estimated_tokens;
        
        // 存储更新后的用户数据
        let user_json = json!({
            "balance": new_balance,
        });

        self.nexus_db.insert("users", user_id, user_json)
            .map_err(IamError::DatabaseError)?;

        Ok(())
    }
}
