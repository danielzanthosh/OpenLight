use crate::tools::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct MemoryManager {
    path: PathBuf,
    cache: Mutex<HashMap<String, String>>,
}

impl MemoryManager {
    pub fn new(path: Option<PathBuf>) -> Self {
        let p = path.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".openlight")
                .join("memory.json")
        });

        let mut map = HashMap::new();
        if p.exists() {
            if let Ok(content) = fs::read_to_string(&p) {
                if let Ok(loaded) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    map = loaded;
                }
            }
        }

        Self {
            path: p,
            cache: Mutex::new(map),
        }
    }

    fn save(&self) -> Result<(), String> {
        let lock = self.cache.lock().map_err(|e| e.to_string())?;
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let serialized = serde_json::to_string_pretty(&*lock).map_err(|e| e.to_string())?;
        fs::write(&self.path, serialized).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn store(&self, key: &str, value: &str) -> Result<String, String> {
        {
            let mut lock = self.cache.lock().map_err(|e| e.to_string())?;
            lock.insert(key.to_string(), value.to_string());
        }
        self.save()?;
        Ok(format!("Stored memory: '{key}'"))
    }

    pub fn recall(&self, query: &str) -> Vec<(String, String)> {
        let q_lower = query.to_lowercase();
        let lock = self.cache.lock().unwrap();
        lock.iter()
            .filter(|(k, v)| k.to_lowercase().contains(&q_lower) || v.to_lowercase().contains(&q_lower))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn list_all(&self) -> HashMap<String, String> {
        let lock = self.cache.lock().unwrap();
        lock.clone()
    }

    pub fn delete(&self, key: &str) -> Result<String, String> {
        let removed = {
            let mut lock = self.cache.lock().map_err(|e| e.to_string())?;
            lock.remove(key)
        };
        if removed.is_some() {
            self.save()?;
            Ok(format!("Deleted memory: '{key}'"))
        } else {
            Ok(format!("Key '{key}' not found in memory"))
        }
    }

    pub fn format_for_prompt(&self) -> String {
        let all = self.list_all();
        if all.is_empty() {
            return String::new();
        }
        let mut lines = vec!["--- Persistent Memories ---".to_string()];
        for (k, v) in all {
            lines.push(format!("- {k}: {v}"));
        }
        lines.join("\n")
    }
}

pub struct MemoryTool {
    manager: std::sync::Arc<MemoryManager>,
}

impl MemoryTool {
    pub fn new(manager: std::sync::Arc<MemoryManager>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl Tool for MemoryTool {
    fn name(&self) -> &str {
        "memory"
    }

    fn description(&self) -> &str {
        "Store, recall, list, or delete long-term persistent memories."
    }

    fn schema(&self) -> Value {
        json!({
            "name": self.name(),
            "description": self.description(),
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["store", "recall", "list", "delete"],
                        "description": "Memory operation."
                    },
                    "key": { "type": "string", "description": "Key name." },
                    "value": { "type": "string", "description": "Value to store." },
                    "query": { "type": "string", "description": "Search query for recall." }
                },
                "required": ["action"]
            }
        })
    }

    async fn execute(&self, arguments: Value) -> ToolResult {
        let action = arguments["action"].as_str().unwrap_or("list");
        match action {
            "store" => {
                let key = arguments["key"].as_str().unwrap_or("");
                let val = arguments["value"].as_str().unwrap_or("");
                if key.is_empty() || val.is_empty() {
                    return ToolResult::err("Both 'key' and 'value' required for store.");
                }
                match self.manager.store(key, val) {
                    Ok(msg) => ToolResult::ok(msg),
                    Err(e) => ToolResult::err(e),
                }
            }
            "recall" => {
                let query = arguments["query"].as_str().unwrap_or("");
                let results = self.manager.recall(query);
                if results.is_empty() {
                    ToolResult::ok(format!("No memories found matching '{query}'."))
                } else {
                    let formatted: Vec<String> = results.iter().map(|(k, v)| format!("- {k}: {v}")).collect();
                    ToolResult::ok(format!("Memories:\n{}", formatted.join("\n")))
                }
            }
            "list" => {
                let all = self.manager.list_all();
                if all.is_empty() {
                    ToolResult::ok("Memory is currently empty.")
                } else {
                    let formatted: Vec<String> = all.iter().map(|(k, v)| format!("- {k}: {v}")).collect();
                    ToolResult::ok(format!("Memories:\n{}", formatted.join("\n")))
                }
            }
            "delete" => {
                let key = arguments["key"].as_str().unwrap_or("");
                match self.manager.delete(key) {
                    Ok(msg) => ToolResult::ok(msg),
                    Err(e) => ToolResult::err(e),
                }
            }
            other => ToolResult::err(format!("Unknown memory action: '{other}'")),
        }
    }
}
