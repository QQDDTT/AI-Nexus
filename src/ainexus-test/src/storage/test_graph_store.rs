use ai_nexus::core::interfaces::{GraphEdge, GraphNode, GraphStorage, NodeLabel};
use ai_nexus::storage::graph::PetGraphStore;
use tempfile::tempdir;

/// 测试图谱节点写入与邻居查询
#[tokio::test]
async fn test_graph_node_upsert_and_neighbors() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("graph_store.bin");
    let store = PetGraphStore::new(&path)?;

    // 插入三个节点
    let n1 = GraphNode {
        id: "skill_code_review".to_string(),
        label: NodeLabel::Skill,
        properties: b"Code review skill".to_vec(),
        embedding: vec![1.0, 0.0, 0.0],
    };
    let n2 = GraphNode {
        id: "skill_refactor".to_string(),
        label: NodeLabel::Skill,
        properties: b"Refactor skill".to_vec(),
        embedding: vec![0.9, 0.1, 0.0],
    };
    let n3 = GraphNode {
        id: "concept_clean_code".to_string(),
        label: NodeLabel::Concept,
        properties: b"Clean Code philosophy".to_vec(),
        embedding: vec![0.8, 0.2, 0.0],
    };

    store.upsert_node(n1).await?;
    store.upsert_node(n2).await?;
    store.upsert_node(n3).await?;

    // 建立边：code_review -> refactor (weight=0.8)
    store
        .upsert_edge(
            "skill_code_review",
            "skill_refactor",
            GraphEdge {
                relation: "triggers".to_string(),
                weight: 0.8,
            },
        )
        .await?;

    // 建立边：refactor -> clean_code (weight=0.7)
    store
        .upsert_edge(
            "skill_refactor",
            "concept_clean_code",
            GraphEdge {
                relation: "relates_to".to_string(),
                weight: 0.7,
            },
        )
        .await?;

    // 验证邻居查询
    let neighbors = store.neighbors("skill_code_review").await?;
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].id, "skill_refactor");

    Ok(())
}

/// 测试激活扩散（Spreading Activation）联想记忆
#[tokio::test]
async fn test_spreading_activation() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("graph_store_sa.bin");
    let store = PetGraphStore::new(&path)?;

    // 构建一个简单的知识网络：
    //   memory_nick_likes_rust -> concept_rust (weight=0.9)
    //   concept_rust -> skill_rust_coding (weight=0.8)
    //   skill_rust_coding -> concept_performance (weight=0.7)
    let nodes = vec![
        GraphNode { id: "memory_nick_likes_rust".to_string(), label: NodeLabel::Memory, properties: b"Nick loves Rust".to_vec(), embedding: vec![] },
        GraphNode { id: "concept_rust".to_string(), label: NodeLabel::Concept, properties: b"Rust programming language".to_vec(), embedding: vec![] },
        GraphNode { id: "skill_rust_coding".to_string(), label: NodeLabel::Skill, properties: b"Write Rust code".to_vec(), embedding: vec![] },
        GraphNode { id: "concept_performance".to_string(), label: NodeLabel::Concept, properties: b"High performance computing".to_vec(), embedding: vec![] },
    ];

    for node in nodes {
        store.upsert_node(node).await?;
    }

    let edges = vec![
        ("memory_nick_likes_rust", "concept_rust", 0.9),
        ("concept_rust", "skill_rust_coding", 0.8),
        ("skill_rust_coding", "concept_performance", 0.7),
    ];

    for (from, to, weight) in edges {
        store
            .upsert_edge(from, to, GraphEdge { relation: "relates_to".to_string(), weight })
            .await?;
    }

    // 以 memory_nick_likes_rust 为种子，激活扩散 depth=3
    let seeds = vec!["memory_nick_likes_rust".to_string()];
    let results = store.spreading_activation(&seeds, 5, 3).await?;

    // 应该激活 3 个节点（concept_rust, skill_rust_coding, concept_performance）
    assert_eq!(results.len(), 3, "应激活 3 个关联节点");

    // 激活值应按衰减排序，最近的 concept_rust 应排第一
    assert_eq!(results[0].node.id, "concept_rust");
    assert!(results[0].activation > results[1].activation, "激活值应递减");

    println!("激活扩散结果:");
    for r in &results {
        println!("  {} (activation={:.4})", r.node.id, r.activation);
    }

    Ok(())
}

/// 测试图谱节点持久化（写入后重新加载）
#[tokio::test]
async fn test_graph_persistence() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("graph_persist.bin");

    // 第一次写入
    {
        let store = PetGraphStore::new(&path)?;
        store
            .upsert_node(GraphNode {
                id: "entity_test".to_string(),
                label: NodeLabel::Entity,
                properties: b"Persistent entity".to_vec(),
                embedding: vec![1.0, 2.0, 3.0],
            })
            .await?;
        // flush 是异步后台任务，稍等确保写入
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // 第二次加载，验证节点仍存在
    {
        let store = PetGraphStore::new(&path)?;
        let by_label = store.get_nodes_by_label(&NodeLabel::Entity).await?;
        assert_eq!(by_label.len(), 1);
        assert_eq!(by_label[0].id, "entity_test");
        assert_eq!(by_label[0].embedding, vec![1.0, 2.0, 3.0]);
    }

    Ok(())
}
