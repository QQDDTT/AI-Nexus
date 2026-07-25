use ai_nexus::iam::{IamGateway, IamError};
use ai_nexus::storage::NexusDb;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_iam_quota_deduction_and_exhaustion() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let nexus_db = Arc::new(NexusDb::new(dir.path())?);
    let iam = IamGateway::new(nexus_db.clone());

    let user_id = "test_user_001";

    // Request 1: Takes 2000 (Balance drops from 5000 to 3000)
    iam.verify_and_deduct_quota(user_id, 2000).await?;
    
    // Request 2: Takes 2500 (Balance drops from 3000 to 500)
    iam.verify_and_deduct_quota(user_id, 2500).await?;
    
    // Request 3: Takes 1000 (Balance 500 < 1000) -> Should fail
    let result = iam.verify_and_deduct_quota(user_id, 1000).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        IamError::QuotaExceeded { required, balance } => {
            assert_eq!(required, 1000);
            assert_eq!(balance, 500);
        }
        _ => panic!("Expected QuotaExceeded error"),
    }

    Ok(())
}
