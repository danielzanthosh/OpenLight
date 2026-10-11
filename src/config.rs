use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLightConfig {
    pub default_provider: String,
    pub openai_model: String,
    pub anthropic_model: String,
    pub gemini_model: String,
    pub openrouter_model: String,
    pub ollama_model: String,
    pub ollama_base_url: String,

    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub openrouter_api_key: Option<String>,

    pub max_turns: usize,
    pub default_monitor: usize,
    pub working_dir: String,
}

impl Default for OpenLightConfig {
    fn default() -> Self {
        Self {
            default_provider: "gemini".to_string(),
            openai_model: "gpt-4o".to_string(),
            anthropic_model: "claude-3-7-sonnet-20250219".to_string(),
            gemini_model: "gemini-2.5-pro".to_string(),
            openrouter_model: "anthropic/claude-3.7-sonnet".to_string(),
            ollama_model: "llama3.2-vision".to_string(),
            ollama_base_url: "http://localhost:11434/v1".to_string(),

            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            gemini_api_key: std::env::var("GEMINI_API_KEY").ok(),
            openrouter_api_key: std::env::var("OPENROUTER_API_KEY").ok(),

            max_turns: 30,
            default_monitor: 1,
            working_dir: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .to_string_lossy()
                .to_string(),
        }
    }
}

impl OpenLightConfig {
    pub fn config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".openlight").join("config.json")
    }

    pub fn load() -> Self {
        let mut cfg = Self::default();
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(loaded) = serde_json::from_str::<OpenLightConfig>(&content) {
                    cfg = loaded;
                }
            }
        }

        // Environment variables always override config file
        if let Ok(k) = std::env::var("OPENAI_API_KEY") {
            cfg.openai_api_key = Some(k);
        }
        if let Ok(k) = std::env::var("ANTHROPIC_API_KEY") {
            cfg.anthropic_api_key = Some(k);
        }
        if let Ok(k) = std::env::var("GEMINI_API_KEY") {
            cfg.gemini_api_key = Some(k);
        }
        if let Ok(k) = std::env::var("OPENROUTER_API_KEY") {
            cfg.openrouter_api_key = Some(k);
        }

        cfg
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
