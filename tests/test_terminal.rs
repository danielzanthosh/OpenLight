use openlight::tools::terminal::TerminalTool;
use openlight::tools::Tool;
use serde_json::json;

#[tokio::test]
async fn test_terminal_simple_command() {
    let tool = TerminalTool::new(None);
    let res = tool
        .execute(json!({
            "command": "Write-Output 'HelloOpenLightRust'"
        }))
        .await;
    assert!(res.success);
    assert!(res.output.contains("HelloOpenLightRust"));
}

#[tokio::test]
async fn test_terminal_timeout() {
    let tool = TerminalTool::new(None);
    let res = tool
        .execute(json!({
            "command": "Start-Sleep -Seconds 3",
            "timeout_seconds": 1
        }))
        .await;
    assert!(!res.success);
    assert!(res.output.to_lowercase().contains("timed out"));
}
