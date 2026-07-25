use ai_nexus::storage::vector::{Hnswai_nexus::core::interfaces::VectorStore, ai_nexus::core::interfaces::VectorStore};
use anyhow::Result;
use tempfile::tempdir;

#[tokio::test]
async fn test_vector_store_persistence_and_search() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("vector_store.bin");

    // Scope for writing
    {
        let store = Hnswai_nexus::core::interfaces::VectorStore::new(&path)?;
        
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0];
        let v3 = vec![0.707, 0.707, 0.0]; // close to both, but let's query [1,0,0]

        store.upsert("skills", "skill_1", v1.clone(), b"doc1".to_vec()).await?;
        store.upsert("skills", "skill_2", v2.clone(), b"doc2".to_vec()).await?;
        store.upsert("skills", "skill_3", v3.clone(), b"doc3".to_vec()).await?;
        
        // Wait a bit to ensure async flush completes
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    // Reopen from disk
    {
        let store = Hnswai_nexus::core::interfaces::VectorStore::new(&path)?;
        let query = vec![1.0, 0.1, 0.0]; // Should be closest to skill_1
        
        let results = store.search("skills", &query, 2).await?;
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "skill_1");
        assert_eq!(results[1].id, "skill_3");
    }

    Ok(())
}
