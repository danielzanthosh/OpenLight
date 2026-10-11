pub mod filesystem;
pub mod terminal;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub image_base64: Option<String>,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            image_base64: None,
        }
    }

    pub fn err(output: impl Into<String>) -> Self {
        Self {
            success: false,
            output: output.into(),
            image_base64: None,
        }
    }

    pub fn with_image(output: impl Into<String>, image_base64: String) -> Self {
        Self {
            success: true,
            output: output.into(),
            image_base64: Some(image_base64),
        }
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> Value;
    async fn execute(&self, arguments: Value) -> ToolResult;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn list_schemas(&self) -> Vec<Value> {
        self.tools.values().map(|t| t.schema()).collect()
    }

    pub async fn execute_tool(&self, name: &str, arguments: Value) -> ToolResult {
        if let Some(tool) = self.tools.get(name) {
            tool.execute(arguments).await
        } else {
            ToolResult::err(format!("Tool '{name}' not found in registry."))
        }
    }

    pub fn create_default(working_dir: &str) -> Self {
        let mut registry = Self::new();
        registry.register(Arc::new(terminal::TerminalTool::new(Some(
            working_dir.to_string(),
        ))));
        registry.register(Arc::new(filesystem::ReadFileTool));
        registry.register(Arc::new(filesystem::WriteFileTool));
        registry.register(Arc::new(filesystem::PatchFileTool));
        registry.register(Arc::new(filesystem::SearchFilesTool));
        registry
    }
}
