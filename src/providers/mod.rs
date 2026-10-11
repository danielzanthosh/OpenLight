pub mod anthropic;
pub mod gemini;
pub mod openai;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMMessage {
    pub role: String, // "system", "user", "assistant", "tool"
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
    pub image_base64: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct LLMResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: Option<String>,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn generate_turn(
        &self,
        messages: &[LLMMessage],
        tools_schema: &[Value],
        system_instruction: Option<&str>,
    ) -> Result<LLMResponse, Box<dyn std::error::Error + Send + Sync>>;

    fn provider_name(&self) -> &str;
    fn model_name(&self) -> &str;
}

pub fn create_provider(
    provider_name: Option<&str>,
    model_override: Option<&str>,
    config: &crate::config::OpenLightConfig,
) -> Result<Box<dyn LLMProvider>, Box<dyn std::error::Error + Send + Sync>> {
    let name = provider_name.unwrap_or(&config.default_provider).to_lowercase();
    match name.as_str() {
        "gemini" => {
            let model = model_override.unwrap_or(&config.gemini_model);
            let key = config
                .gemini_api_key
                .clone()
                .ok_or("GEMINI_API_KEY is not set.")?;
            Ok(Box::new(gemini::GeminiProvider::new(model, &key)))
        }
        "openai" => {
            let model = model_override.unwrap_or(&config.openai_model);
            let key = config
                .openai_api_key
                .clone()
                .ok_or("OPENAI_API_KEY is not set.")?;
            Ok(Box::new(openai::OpenAIProvider::new(
                model,
                &key,
                "https://api.openai.com/v1",
            )))
        }
        "anthropic" => {
            let model = model_override.unwrap_or(&config.anthropic_model);
            let key = config
                .anthropic_api_key
                .clone()
                .ok_or("ANTHROPIC_API_KEY is not set.")?;
            Ok(Box::new(anthropic::AnthropicProvider::new(model, &key)))
        }
        "openrouter" => {
            let model = model_override.unwrap_or(&config.openrouter_model);
            let key = config
                .openrouter_api_key
                .clone()
                .ok_or("OPENROUTER_API_KEY is not set.")?;
            Ok(Box::new(openai::OpenAIProvider::new(
                model,
                &key,
                "https://openrouter.ai/api/v1",
            )))
        }
        "ollama" => {
            let model = model_override.unwrap_or(&config.ollama_model);
            Ok(Box::new(openai::OpenAIProvider::new(
                model,
                "ollama",
                &config.ollama_base_url,
            )))
        }
        other => Err(format!(
            "Unsupported provider '{other}'. Supported: gemini, openai, anthropic, openrouter, ollama"
        )
        .into()),
    }
}
