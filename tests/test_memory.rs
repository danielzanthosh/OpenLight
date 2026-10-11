use openlight::agent::memory::{MemoryManager, MemoryTool};
use openlight::tools::Tool;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_memory_crud() {
    let tmp = std::env::temp_dir().join("openlight_mem_test").join("mem.json");
    let mgr = Arc::new(MemoryManager::new(Some(tmp.clone())));

    mgr.store("theme", "dark").unwrap();
    mgr.store("lang", "Rust").unwrap();

    let recalled = mgr.recall("rust");
    assert_eq!(recalled.len(), 1);
    assert_eq!(recalled[0].0, "lang");
    assert_eq!(recalled[0].1, "Rust");

    let all = mgr.list_all();
    assert_eq!(all.len(), 2);

    mgr.delete("theme").unwrap();
    assert_eq!(mgr.list_all().len(), 1);

    let _ = std::fs::remove_file(&tmp);
}

#[tokio::test]
async fn test_memory_tool() {
    let tmp = std::env::temp_dir().join("openlight_tool_test").join("mem.json");
    let mgr = Arc::new(MemoryManager::new(Some(tmp.clone())));
    let tool = MemoryTool::new(mgr);

    let s_res = tool
        .execute(json!({
            "action": "store",
            "key": "editor",
            "value": "Neovim"
        }))
        .await;
    assert!(s_res.success);

    let r_res = tool
        .execute(json!({
            "action": "recall",
            "query": "Neovim"
        }))
        .await;
    assert!(r_res.success);
    assert!(r_res.output.contains("Neovim"));

    let _ = std::fs::remove_file(&tmp);
}
