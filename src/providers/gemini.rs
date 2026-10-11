use super::{LLMMessage, LLMProvider, LLMResponse, ToolCall};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

pub struct GeminiProvider {
    pub model: String,
    pub api_key: String,
    client: Client,
}

impl GeminiProvider {
    pub fn new(model: &str, api_key: &str) -> Self {
        Self {
            model: model.to_string(),
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    fn provider_name(&self) -> &str {
        "gemini"
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    async fn generate_turn(
        &self,
        messages: &[LLMMessage],
        tools_schema: &[Value],
        system_instruction: Option<&str>,
    ) -> Result<LLMResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut contents = Vec::new();

        for msg in messages {
            let role = if msg.role == "assistant" { "model" } else { "user" };
            let mut parts = Vec::new();

            if msg.role == "tool" {
                parts.push(json!({
                    "functionResponse": {
                        "name": msg.tool_call_id.clone().unwrap_or_else(|| "function".to_string()),
                        "response": {
                            "output": msg.content
                        }
                    }
                }));
                contents.push(json!({
                    "role": "user",
                    "parts": parts
                }));
                continue;
            }

            if !msg.content.is_empty() {
                parts.push(json!({ "text": msg.content }));
            }

            if let Some(ref b64) = msg.image_base64 {
                parts.push(json!({
                    "inline_data": {
                        "mime_type": "image/jpeg",
                        "data": b64
                    }
                }));
            }

            if let Some(ref tcs) = msg.tool_calls {
                for tc in tcs {
                    parts.push(json!({
                        "functionCall": {
                            "name": tc.name,
                            "args": tc.arguments
                        }
                    }));
                }
            }

            if !parts.is_empty() {
                contents.push(json!({
                    "role": role,
                    "parts": parts
                }));
            }
        }

        let mut payload = json!({
            "contents": contents
        });

        if let Some(sys) = system_instruction {
            payload["system_instruction"] = json!({
                "parts": [{"text": sys}]
            });
        }

        if !tools_schema.is_empty() {
            let func_decls: Vec<Value> = tools_schema
                .iter()
                .map(|s| {
                    json!({
                        "name": s["name"],
                        "description": s["description"],
                        "parameters": s["parameters"]
                    })
                })
                .collect();
            payload["tools"] = json!([{
                "function_declarations": func_decls
            }]);
        }

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Gemini API error: {err_text}").into());
        }

        let resp_json: Value = resp.json().await?;
        let mut content = String::new();
        let mut tool_calls = Vec::new();

        if let Some(candidates) = resp_json["candidates"].as_array() {
            if let Some(cand) = candidates.first() {
                if let Some(parts) = cand["content"]["parts"].as_array() {
                    for p in parts {
                        if let Some(t) = p["text"].as_str() {
                            content.push_str(t);
                        }
                        if let Some(fc) = p.get("functionCall") {
                            let name = fc["name"].as_str().unwrap_or("").to_string();
                            let args = fc["args"].clone();
                            tool_calls.push(ToolCall {
                                id: name.clone(),
                                name,
                                arguments: args,
                            });
                        }
                    }
                }
            }
        }

        Ok(LLMResponse {
            content,
            tool_calls,
            finish_reason: Some("stop".to_string()),
        })
    }
}
