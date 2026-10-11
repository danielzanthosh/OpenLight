use crate::tools::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,
    pub path: PathBuf,
}

pub struct SkillsManager {
    search_dirs: Vec<PathBuf>,
}

impl SkillsManager {
    pub fn new(custom_dirs: Option<Vec<PathBuf>>) -> Self {
        let default_dirs = vec![
            PathBuf::from("./skills"),
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".openlight")
                .join("skills"),
        ];
        Self {
            search_dirs: custom_dirs.unwrap_or(default_dirs),
        }
    }

    pub fn discover_skills(&self) -> Vec<Skill> {
        let mut list = Vec::new();
        for dir in &self.search_dirs {
            if !dir.exists() {
                continue;
            }
            self.scan_recursive(dir, &mut list);
        }
        list
    }

    fn scan_recursive(&self, path: &Path, list: &mut Vec<Skill>) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    self.scan_recursive(&p, list);
                } else if p.is_file() {
                    if let Some(file_name) = p.file_name().and_then(|n| n.to_str()) {
                        let lower = file_name.to_lowercase();
                        if lower == "skill.md" || lower.ends_with(".skill.md") {
                            if let Some(skill) = self.parse_skill_file(&p) {
                                list.push(skill);
                            }
                        }
                    }
                }
            }
        }
    }

    fn parse_skill_file(&self, path: &Path) -> Option<Skill> {
        let raw = fs::read_to_string(path).ok()?;
        let mut name = path.file_stem()?.to_string_lossy().to_string();
        let mut description = String::new();
        let mut content = raw.clone();

        if raw.starts_with("---") {
            let parts: Vec<&str> = raw.splitn(3, "---").collect();
            if parts.len() >= 3 {
                let frontmatter = parts[1];
                content = parts[2].trim().to_string();
                for line in frontmatter.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("name:") {
                        name = trimmed.trim_start_matches("name:").trim().trim_matches('"').trim_matches('\'').to_string();
                    } else if trimmed.starts_with("description:") {
                        description = trimmed
                            .trim_start_matches("description:")
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();
                    }
                }
            }
        }

        Some(Skill {
            name,
            description,
            content,
            path: path.to_path_buf(),
        })
    }

    pub fn create_skill(&self, name: &str, description: &str, instructions: &str) -> Result<String, String> {
        let target_dir = self.search_dirs.first().cloned().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".openlight")
                .join("skills")
        });

        let slug: String = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();

        let skill_folder = target_dir.join(slug);
        let _ = fs::create_dir_all(&skill_folder);
        let skill_file = skill_folder.join("SKILL.md");

        let body = format!(
            "---\nname: {name}\ndescription: \"{description}\"\n---\n\n# {name}\n\n{instructions}\n"
        );

        fs::write(&skill_file, body).map_err(|e| e.to_string())?;
        Ok(format!("Successfully created skill '{name}' at {}", skill_file.display()))
    }
}

pub struct SkillTool {
    manager: Arc<SkillsManager>,
}

impl SkillTool {
    pub fn new(manager: Arc<SkillsManager>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl Tool for SkillTool {
    fn name(&self) -> &str {
        "skill_manage"
    }

    fn description(&self) -> &str {
        "Discover, view, or create agent skills."
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
                        "enum": ["list", "view", "create"],
                        "description": "Skill operation."
                    },
                    "name": { "type": "string", "description": "Skill name for view or create." },
                    "description": { "type": "string", "description": "Brief description for create." },
                    "instructions": { "type": "string", "description": "Instructions for create." }
                },
                "required": ["action"]
            }
        })
    }

    async fn execute(&self, arguments: Value) -> ToolResult {
        let action = arguments["action"].as_str().unwrap_or("list");
        match action {
            "list" => {
                let skills = self.manager.discover_skills();
                if skills.is_empty() {
                    ToolResult::ok("No specialized skills currently discovered.")
                } else {
                    let lines: Vec<String> = skills
                        .iter()
                        .map(|s| format!("- **{}**: {}", s.name, s.description))
                        .collect();
                    ToolResult::ok(format!("Available Skills:\n{}", lines.join("\n")))
                }
            }
            "view" => {
                let name = arguments["name"].as_str().unwrap_or("");
                let skills = self.manager.discover_skills();
                for s in skills {
                    if s.name.eq_ignore_ascii_case(name) {
                        return ToolResult::ok(format!("=== Skill: {} ===\n{}", s.name, s.content));
                    }
                }
                ToolResult::err(format!("Skill '{name}' not found."))
            }
            "create" => {
                let name = arguments["name"].as_str().unwrap_or("");
                let desc = arguments["description"].as_str().unwrap_or("");
                let inst = arguments["instructions"].as_str().unwrap_or("");
                if name.is_empty() || inst.is_empty() {
                    return ToolResult::err("Both 'name' and 'instructions' required for create.");
                }
                match self.manager.create_skill(name, desc, inst) {
                    Ok(msg) => ToolResult::ok(msg),
                    Err(e) => ToolResult::err(e),
                }
            }
            other => ToolResult::err(format!("Unknown skill action: '{other}'")),
        }
    }
}
