use openlight::agent::skills::{SkillsManager, SkillTool};
use openlight::tools::Tool;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_skill_create_and_view() {
    let tmp = std::env::temp_dir().join("openlight_skills_test");
    let _ = std::fs::create_dir_all(&tmp);

    let mgr = Arc::new(SkillsManager::new(Some(vec![tmp.clone()])));
    let tool = SkillTool::new(mgr);

    let c_res = tool
        .execute(json!({
            "action": "create",
            "name": "Rust Testing",
            "description": "Guidelines for writing cargo tests",
            "instructions": "Always assert outcomes clearly."
        }))
        .await;
    assert!(c_res.success);

    let v_res = tool
        .execute(json!({
            "action": "view",
            "name": "Rust Testing"
        }))
        .await;
    assert!(v_res.success);
    assert!(v_res.output.contains("assert outcomes"));

    let _ = std::fs::remove_dir_all(&tmp);
}
