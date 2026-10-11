use super::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::process::Command;

pub struct TerminalTool {
    pub default_cwd: Option<String>,
}

impl TerminalTool {
    pub fn new(default_cwd: Option<String>) -> Self {
        Self { default_cwd }
    }
}

#[async_trait]
impl Tool for TerminalTool {
    fn name(&self) -> &str {
        "terminal"
    }

    fn description(&self) -> &str {
        "Execute shell/terminal commands (PowerShell on Windows, Bash on Linux/macOS)."
    }

    fn schema(&self) -> Value {
        json!({
            "name": self.name(),
            "description": self.description(),
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The command string to execute in the shell."
                    },
                    "timeout_seconds": {
                        "type": "integer",
                        "description": "Timeout in seconds (default: 60)."
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Working directory for execution."
                    }
                },
                "required": ["command"]
            }
        })
    }

    async fn execute(&self, arguments: Value) -> ToolResult {
        let command_str = match arguments["command"].as_str() {
            Some(c) if !c.is_empty() => c,
            _ => return ToolResult::err("Missing or empty 'command' argument."),
        };

        let timeout_secs = arguments["timeout_seconds"].as_u64().unwrap_or(60);
        let cwd = arguments["cwd"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| self.default_cwd.clone());

        #[cfg(windows)]
        let mut cmd = Command::new("powershell");
        #[cfg(windows)]
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", command_str]);

        #[cfg(not(windows))]
        let mut cmd = Command::new("/bin/bash");
        #[cfg(not(windows))]
        cmd.args(["-c", command_str]);

        if let Some(ref dir) = cwd {
            cmd.current_dir(dir);
        }

        let child = match cmd.output() {
            _ => {
                let execution = tokio::time::timeout(Duration::from_secs(timeout_secs), cmd.output());
                match execution.await {
                    Ok(Ok(output)) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let mut combined = String::new();
                        if !stdout.is_empty() {
                            combined.push_str(&stdout);
                        }
                        if !stderr.is_empty() {
                            if !combined.is_empty() {
                                combined.push_str("\n--- STDERR ---\n");
                            }
                            combined.push_str(&stderr);
                        }
                        if combined.is_empty() {
                            combined = format!("(Command exited with code {})", output.status);
                        }
                        if output.status.success() {
                            ToolResult::ok(combined)
                        } else {
                            ToolResult::err(combined)
                        }
                    }
                    Ok(Err(e)) => ToolResult::err(format!("Process spawn error: {e}")),
                    Err(_) => ToolResult::err(format!("Command timed out after {timeout_secs}s.")),
                }
            }
        };

        child
    }
}
