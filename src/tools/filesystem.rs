use super::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read file content with optional line ranges (1-indexed)."
    }

    fn schema(&self) -> Value {
        json!({
            "name": self.name(),
            "description": self.description(),
            "parameters": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to the file to read." },
                    "start_line": { "type": "integer", "description": "1-indexed starting line number." },
                    "end_line": { "type": "integer", "description": "1-indexed ending line number." }
                },
                "required": ["path"]
            }
        })
    }

    async fn execute(&self, arguments: Value) -> ToolResult {
        let path_str = match arguments["path"].as_str() {
            Some(p) => p,
            None => return ToolResult::err("Missing 'path' argument."),
        };

        match fs::read_to_string(path_str) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let total = lines.len();
                let start = arguments["start_line"].as_u64().unwrap_or(1) as usize;
                let end = arguments["end_line"]
                    .as_u64()
                    .map(|e| e as usize)
                    .unwrap_or(total);

                let start_idx = start.saturating_sub(1);
                let end_idx = end.min(total);

                if start_idx >= total {
                    return ToolResult::ok(format!("File: {path_str} (Empty range beyond total lines: {total})"));
                }

                let slice = &lines[start_idx..end_idx];
                let formatted: String = slice
                    .iter()
                    .enumerate()
                    .map(|(idx, line)| format!("{}: {line}\n", start_idx + idx + 1))
                    .collect();

                ToolResult::ok(format!("File: {path_str} (Lines {start}-{end} of {total}):\n{formatted}"))
            }
            Err(e) => ToolResult::err(format!("Error reading {path_str}: {e}")),
        }
    }
}

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Create or overwrite a file on disk."
    }

    fn schema(&self) -> Value {
        json!({
            "name": self.name(),
            "description": self.description(),
            "parameters": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Target file path." },
                    "content": { "type": "string", "description": "Content to write into the file." },
                    "overwrite": { "type": "boolean", "description": "Whether to overwrite existing file (default: true)." }
                },
                "required": ["path", "content"]
            }
        })
    }

    async fn execute(&self, arguments: Value) -> ToolResult {
        let path_str = match arguments["path"].as_str() {
            Some(p) => p,
            None => return ToolResult::err("Missing 'path' argument."),
        };
        let content = match arguments["content"].as_str() {
            Some(c) => c,
            None => return ToolResult::err("Missing 'content' argument."),
        };
        let overwrite = arguments["overwrite"].as_bool().unwrap_or(true);

        let path = Path::new(path_str);
        if path.exists() && !overwrite {
            return ToolResult::err(format!("File '{path_str}' exists and overwrite is false."));
        }

        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return ToolResult::err(format!("Error creating parent directories: {e}"));
            }
        }

        match fs::write(path, content) {
            Ok(_) => ToolResult::ok(format!("Successfully wrote {} bytes to {path_str}", content.len())),
            Err(e) => ToolResult::err(format!("Error writing to {path_str}: {e}")),
        }
    }
}

pub struct PatchFileTool;

#[async_trait]
impl Tool for PatchFileTool {
    fn name(&self) -> &str {
        "patch_file"
    }

    fn description(&self) -> &str {
        "Replace an exact target block of text in an existing file with replacement text."
    }

    fn schema(&self) -> Value {
        json!({
            "name": self.name(),
            "description": self.description(),
            "parameters": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path to patch." },
                    "target_content": { "type": "string", "description": "Exact text chunk in the file to replace." },
                    "replacement_content": { "type": "string", "description": "New text chunk to replace it with." }
                },
                "required": ["path", "target_content", "replacement_content"]
            }
        })
    }

    async fn execute(&self, arguments: Value) -> ToolResult {
        let path_str = match arguments["path"].as_str() {
            Some(p) => p,
            None => return ToolResult::err("Missing 'path' argument."),
        };
        let target = match arguments["target_content"].as_str() {
            Some(t) => t,
            None => return ToolResult::err("Missing 'target_content' argument."),
        };
        let replacement = match arguments["replacement_content"].as_str() {
            Some(r) => r,
            None => return ToolResult::err("Missing 'replacement_content' argument."),
        };

        match fs::read_to_string(path_str) {
            Ok(content) => {
                if !content.contains(target) {
                    return ToolResult::err(format!("Target content not found in {path_str}."));
                }
                let occurrences = content.matches(target).count();
                if occurrences > 1 {
                    return ToolResult::err(format!(
                        "Target content appears {occurrences} times in {path_str}. Provide more surrounding lines for uniqueness."
                    ));
                }

                let new_content = content.replacen(target, replacement, 1);
                match fs::write(path_str, new_content) {
                    Ok(_) => ToolResult::ok(format!("Successfully patched {path_str}")),
                    Err(e) => ToolResult::err(format!("Error saving patched {path_str}: {e}")),
                }
            }
            Err(e) => ToolResult::err(format!("Error reading {path_str}: {e}")),
        }
    }
}

pub struct SearchFilesTool;

#[async_trait]
impl Tool for SearchFilesTool {
    fn name(&self) -> &str {
        "search_files"
    }

    fn description(&self) -> &str {
        "Search files for a query string."
    }

    fn schema(&self) -> Value {
        json!({
            "name": self.name(),
            "description": self.description(),
            "parameters": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Text substring to search for." },
                    "dir": { "type": "string", "description": "Directory to search in (default: '.')." }
                },
                "required": ["query"]
            }
        })
    }

    async fn execute(&self, arguments: Value) -> ToolResult {
        let query = match arguments["query"].as_str() {
            Some(q) => q,
            None => return ToolResult::err("Missing 'query' argument."),
        };
        let dir = arguments["dir"].as_str().unwrap_or(".");

        let mut matches = Vec::new();
        search_recursive(Path::new(dir), query, &mut matches, 50);

        if matches.is_empty() {
            ToolResult::ok(format!("No matches found for '{query}' in {dir}"))
        } else {
            ToolResult::ok(matches.join("\n"))
        }
    }
}

fn search_recursive(path: &Path, query: &str, matches: &mut Vec<String>, max_matches: usize) {
    if matches.len() >= max_matches {
        return;
    }
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(file_name) = p.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with('.') || file_name == "target" || file_name == "node_modules" {
                    continue;
                }
            }
            if p.is_dir() {
                search_recursive(&p, query, matches, max_matches);
            } else if p.is_file() {
                if let Ok(content) = fs::read_to_string(&p) {
                    for (line_idx, line) in content.lines().enumerate() {
                        if line.contains(query) {
                            matches.push(format!("{}:{}: {}", p.display(), line_idx + 1, line.trim()));
                            if matches.len() >= max_matches {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}
