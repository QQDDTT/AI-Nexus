#![cfg(test)]

use ai_nexus::storage::{BlockStore, LedgerBlock};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_append_and_read_records() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().to_str().unwrap();

    let store = BlockStore::new(data_dir, "test_ledger.bin").unwrap();

    let record1 = LedgerBlock {
        timestamp: 1620000000,
        user_id: "user_alpha".to_string(),
        model: "gemini-2.5-flash".to_string(),
        input_tokens: 100,
        output_tokens: 200,
        est_cost_usd: 0.003,
    };

    let record2 = LedgerBlock {
        timestamp: 1620000010,
        user_id: "user_beta".to_string(),
        model: "gemini-1.5-pro".to_string(),
        input_tokens: 50,
        output_tokens: 100,
        est_cost_usd: 0.005,
    };

    // 追加写入
    store.append_record(&record1).unwrap();
    store.append_record(&record2).unwrap();

    // 零拷贝读取
    let records: Vec<LedgerBlock> = store.read_all_records().unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].user_id, "user_alpha");
    assert_eq!(records[0].model, "gemini-2.5-flash");
    assert_eq!(records[1].user_id, "user_beta");
    assert_eq!(records[1].input_tokens, 50);

    // 验证底层二进制文件
    let bin_path = format!("{}/test_ledger.bin", data_dir);
    let bytes = fs::read(&bin_path).unwrap();
    assert!(!bytes.is_empty());
}
