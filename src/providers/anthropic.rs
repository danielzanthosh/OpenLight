use super::{LLMMessage, LLMProvider, LLMResponse, ToolCall};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

pub struct AnthropicProvider {
    pub model: String,
    pub api_key: String,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(model: &str, api_key: &str) -> Self {
        Self {
            model: model.to_string(),
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn provider_name(&self) -> &str {
        "anthropic"
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
        let mut formatted_msgs = Vec::new();

        for msg in messages {
            match msg.role.as_str() {
                "tool" => {
                    formatted_msgs.push(json!({
                        "role": "user",
                        "content": [{
                            "type": "tool_result",
                            "tool_use_id": msg.tool_call_id.clone().unwrap_or_default(),
                            "content": msg.content
                        }]
                    }));
                }
                "assistant" => {
                    let mut content_blocks = Vec::new();
                    if !msg.content.is_empty() {
                        content_blocks.push(json!({
                            "type": "text",
                            "text": msg.content
                        }));
                    }
                    if let Some(ref tcs) = msg.tool_calls {
                        for tc in tcs {
                            content_blocks.push(json!({
                                "type": "tool_use",
                                "id": tc.id,
                                "name": tc.name,
                                "input": tc.arguments
                            }));
                        }
                    }
                    formatted_msgs.push(json!({
                        "role": "assistant",
                        "content": content_blocks
                    }));
                }
                _ => {
                    let mut blocks = Vec::new();
                    if !msg.content.is_empty() {
                        blocks.push(json!({
                            "type": "text",
                            "text": msg.content
                        }));
                    }
                    if let Some(ref b64) = msg.image_base64 {
                        blocks.push(json!({
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": "image/jpeg",
                                "data": b64
                            }
                        }));
                    }
                    formatted_msgs.push(json!({
                        "role": "user",
                        "content": blocks
                    }));
                }
            }
        }

        let mut payload = json!({
            "model": self.model,
            "messages": formatted_msgs,
            "max_tokens": 4096
        });

        if let Some(sys) = system_instruction {
            payload["system"] = json!(sys);
        }

        if !tools_schema.is_empty() {
            let tools: Vec<Value> = tools_schema
                .iter()
                .map(|s| {
                    json!({
                        "name": s["name"],
                        "description": s["description"],
                        "input_schema": s["parameters"]
                    })
                })
                .collect();
            payload["tools"] = Value::Array(tools);
        }

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Anthropic API error: {err_text}").into());
        }

        let resp_json: Value = resp.json().await?;
        let mut content = String::new();
        let mut tool_calls = Vec::new();

        if let Some(blocks) = resp_json["content"].as_array() {
            for b in blocks {
                let b_type = b["type"].as_str().unwrap_or("");
                if b_type == "text" {
                    content.push_str(b["text"].as_str().unwrap_or(""));
                } else if b_type == "tool_use" {
                    let id = b["id"].as_str().unwrap_or("").to_string();
                    let name = b["name"].as_str().unwrap_or("").to_string();
                    let input = b["input"].clone();
                    tool_calls.push(ToolCall {
                        id,
                        name,
                        arguments: input,
                    });
                }
            }
        }

        let stop_reason = resp_json["stop_reason"].as_str().map(|s| s.to_string());

        Ok(LLMResponse {
            content,
            tool_calls,
            finish_reason: stop_reason,
        })
    }
}
