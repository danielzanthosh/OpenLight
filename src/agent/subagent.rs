use crate::tools::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type SubagentTaskFn = Arc<
    dyn Fn(String, String) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>>
        + Send
        + Sync,
>;

pub struct SubagentTool {
    task_fn: SubagentTaskFn,
}

impl SubagentTool {
    pub fn new(task_fn: SubagentTaskFn) -> Self {
        Self { task_fn }
    }
}

#[async_trait]
impl Tool for SubagentTool {
    fn name(&self) -> &str {
        "delegate_task"
    }

    fn description(&self) -> &str {
        "Delegate a subtask or research query to an isolated worker agent."
    }

    fn schema(&self) -> Value {
        json!({
            "name": self.name(),
            "description": self.description(),
            "parameters": {
                "type": "object",
                "properties": {
                    "task": { "type": "string", "description": "Actionable description of the subtask." },
                    "role": { "type": "string", "description": "Role for the subagent worker (e.g. 'Researcher')." }
                },
                "required": ["task"]
            }
        })
    }

    async fn execute(&self, arguments: Value) -> ToolResult {
        let task = arguments["task"].as_str().unwrap_or("");
        let role = arguments["role"].as_str().unwrap_or("Worker");
        if task.is_empty() {
            return ToolResult::err("Missing 'task' argument for delegate_task.");
        }

        match (self.task_fn)(task.to_string(), role.to_string()).await {
            Ok(summary) => ToolResult::ok(format!("[Subagent '{role}' Completed]\n{summary}")),
            Err(e) => ToolResult::err(format!("Subagent error: {e}")),
        }
    }
}
