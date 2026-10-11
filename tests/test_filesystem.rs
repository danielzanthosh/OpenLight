use openlight::tools::filesystem::{PatchFileTool, ReadFileTool, WriteFileTool, SearchFilesTool};
use openlight::tools::Tool;
use serde_json::json;

#[tokio::test]
async fn test_write_and_read_file() {
    let tmp_dir = std::env::temp_dir().join("openlight_test_fs");
    let _ = std::fs::create_dir_all(&tmp_dir);
    let file_path = tmp_dir.join("test.txt");

    let write_tool = WriteFileTool;
    let read_tool = ReadFileTool;

    let res = write_tool
        .execute(json!({
            "path": file_path.to_str().unwrap(),
            "content": "Line 1\nLine 2\nLine 3\n"
        }))
        .await;
    assert!(res.success);

    let read_res = read_tool
        .execute(json!({
            "path": file_path.to_str().unwrap(),
            "start_line": 2,
            "end_line": 2
        }))
        .await;
    assert!(read_res.success);
    assert!(read_res.output.contains("2: Line 2"));

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[tokio::test]
async fn test_patch_file() {
    let tmp_dir = std::env::temp_dir().join("openlight_test_patch");
    let _ = std::fs::create_dir_all(&tmp_dir);
    let file_path = tmp_dir.join("patch.py");
    std::fs::write(&file_path, "def calculate():\n    return 42\n").unwrap();

    let patch_tool = PatchFileTool;
    let res = patch_tool
        .execute(json!({
            "path": file_path.to_str().unwrap(),
            "target_content": "return 42",
            "replacement_content": "return 100"
        }))
        .await;
    assert!(res.success);

    let updated = std::fs::read_to_string(&file_path).unwrap();
    assert!(updated.contains("return 100"));

    let _ = std::fs::remove_dir_all(&tmp_dir);
}
