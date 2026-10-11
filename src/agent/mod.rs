pub mod memory;
pub mod skills;
pub mod subagent;

use crate::computer_use::ComputerUseTool;
use crate::config::OpenLightConfig;
use crate::providers::{create_provider, LLMMessage, LLMProvider};
use crate::tools::ToolRegistry;
use colored::*;
use memory::{MemoryManager, MemoryTool};
use skills::{SkillsManager, SkillTool};
use std::sync::Arc;

pub const HERMES_SYSTEM_PROMPT: &str = r#"You are OpenLight, a high-performance autonomous AI agent built with a native Rust core, real-time desktop screen control (Computer-Using Agent / CUA), and developer tools.

## Core Capabilities
1. **Desktop & Screen Control (CUA)**: You can see the user's screen and interact with their desktop using the `computer_use` tool.
   - Flow: **Capture first** (`action="capture"`) -> inspect the screenshot and active window -> **Actuate** (click coordinates, type text, press keys, scroll) -> **Verify** the updated screen.
   - You can automate desktop applications (browsers, editors, terminals, forms, calculators, games).
2. **Terminal & Developer Automation**: You have a PowerShell/Bash `terminal` tool to run commands, run tests, install packages, and manage processes.
3. **Filesystem Tools**: You can read (`read_file`), write (`write_file`), patch (`patch_file`), and search (`search_files`) files.
4. **Persistent Memory & Skills**: You can persist knowledge and learned workflows across sessions using `memory` and `skill_manage`.
5. **Subagents**: You can delegate focused tasks to isolated workers via `delegate_task`.

## Guidelines
- Proceed methodically step-by-step.
- Verify the outcome of actions. After clicking a button or modifying a file, confirm the state.
- Keep your thoughts concise, focused, and goal-driven.
"#;

pub struct OpenLightAgent {
    pub config: OpenLightConfig,
    pub provider: Box<dyn LLMProvider>,
    pub tool_registry: ToolRegistry,
    pub memory_manager: Arc<MemoryManager>,
    pub skills_manager: Arc<SkillsManager>,
    pub messages: Vec<LLMMessage>,
}

impl OpenLightAgent {
    pub fn new(
        config: OpenLightConfig,
        provider_override: Option<&str>,
        model_override: Option<&str>,
        include_computer_use: bool,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let provider = create_provider(provider_override, model_override, &config)?;
        let mut tool_registry = ToolRegistry::create_default(&config.working_dir);

        let memory_manager = Arc::new(MemoryManager::new(None));
        let skills_manager = Arc::new(SkillsManager::new(None));

        tool_registry.register(Arc::new(MemoryTool::new(memory_manager.clone())));
        tool_registry.register(Arc::new(SkillTool::new(skills_manager.clone())));

        if include_computer_use {
            tool_registry.register(Arc::new(ComputerUseTool::new(1280, config.default_monitor)));
        }

        Ok(Self {
            config,
            provider,
            tool_registry,
            memory_manager,
            skills_manager,
            messages: Vec::new(),
        })
    }

    pub fn build_system_prompt(&self) -> String {
        let mut prompt = HERMES_SYSTEM_PROMPT.to_string();

        let mem_str = self.memory_manager.format_for_prompt();
        if !mem_str.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str(&mem_str);
        }

        let skills = self.skills_manager.discover_skills();
        if !skills.is_empty() {
            prompt.push_str("\n\n--- Available Skills ---\n");
            for s in skills {
                prompt.push_str(&format!("- **{}**: {}\n", s.name, s.description));
            }
        }

        prompt
    }

    pub async fn run_task(&mut self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.messages.push(LLMMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
            tool_calls: None,
            tool_call_id: None,
            image_base64: None,
        });

        let system_instruction = self.build_system_prompt();
        let tools_schema = self.tool_registry.list_schemas();
        let mut final_content = String::new();
        let mut turn = 0;

        while turn < self.config.max_turns {
            turn += 1;

            let response = self
                .provider
                .generate_turn(&self.messages, &tools_schema, Some(&system_instruction))
                .await?;

            if !response.content.is_empty() {
                final_content = response.content.clone();
            }

            self.messages.push(LLMMessage {
                role: "assistant".to_string(),
                content: response.content.clone(),
                tool_calls: if response.tool_calls.is_empty() {
                    None
                } else {
                    Some(response.tool_calls.clone())
                },
                tool_call_id: None,
                image_base64: None,
            });

            if response.tool_calls.is_empty() {
                break;
            }

            for tc in response.tool_calls {
                println!(
                    "{} {} {}",
                    "⚙ Executing:".yellow().bold(),
                    tc.name.bright_white().bold(),
                    tc.arguments.to_string().dimmed()
                );

                let result = self.tool_registry.execute_tool(&tc.name, tc.arguments).await;

                let status_icon = if result.success { "✓".green() } else { "✗".red() };
                let preview = if result.output.len() > 180 {
                    format!("{}...", &result.output[..177])
                } else {
                    result.output.clone()
                };
                println!("{} Tool {}: {}", status_icon, tc.name, preview.dimmed());

                if result.image_base64.is_some() {
                    println!("{}", "📸 Screen snapshot attached to message context.".cyan().dimmed());
                }

                self.messages.push(LLMMessage {
                    role: "tool".to_string(),
                    content: result.output,
                    tool_calls: None,
                    tool_call_id: Some(tc.id),
                    image_base64: result.image_base64,
                });
            }
        }

        Ok(final_content)
    }
}
