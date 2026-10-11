use super::{LLMMessage, LLMProvider, LLMResponse, ToolCall};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

pub struct OpenAIProvider {
    pub model: String,
    pub api_key: String,
    pub base_url: String,
    client: Client,
}

impl OpenAIProvider {
    pub fn new(model: &str, api_key: &str, base_url: &str) -> Self {
        Self {
            model: model.to_string(),
            api_key: api_key.to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn provider_name(&self) -> &str {
        "openai"
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

        if let Some(sys) = system_instruction {
            formatted_msgs.push(json!({
                "role": "system",
                "content": sys
            }));
        }

        for msg in messages {
            match msg.role.as_str() {
                "tool" => {
                    formatted_msgs.push(json!({
                        "role": "tool",
                        "tool_call_id": msg.tool_call_id.clone().unwrap_or_default(),
                        "content": msg.content
                    }));
                }
                "assistant" => {
                    let mut obj = json!({
                        "role": "assistant",
                        "content": msg.content
                    });
                    if let Some(ref tcs) = msg.tool_calls {
                        let calls: Vec<Value> = tcs
                            .iter()
                            .map(|tc| {
                                json!({
                                    "id": tc.id,
                                    "type": "function",
                                    "function": {
                                        "name": tc.name,
                                        "arguments": tc.arguments.to_string()
                                    }
                                })
                            })
                            .collect();
                        obj["tool_calls"] = Value::Array(calls);
                    }
                    formatted_msgs.push(obj);
                }
                _ => {
                    // user role
                    if let Some(ref b64) = msg.image_base64 {
                        let content = json!([
                            {"type": "text", "text": msg.content},
                            {
                                "type": "image_url",
                                "image_url": {
                                    "url": format!("data:image/jpeg;base64,{b64}"),
                                    "detail": "high"
                                }
                            }
                        ]);
                        formatted_msgs.push(json!({
                            "role": "user",
                            "content": content
                        }));
                    } else {
                        formatted_msgs.push(json!({
                            "role": "user",
                            "content": msg.content
                        }));
                    }
                }
            }
        }

        let mut payload = json!({
            "model": self.model,
            "messages": formatted_msgs
        });

        if !tools_schema.is_empty() {
            let tools: Vec<Value> = tools_schema
                .iter()
                .map(|schema| {
                    json!({
                        "type": "function",
                        "function": schema
                    })
                })
                .collect();
            payload["tools"] = Value::Array(tools);
            payload["tool_choice"] = json!("auto");
        }

        let url = format!("{}/chat/completions", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("OpenAI API error: {err_text}").into());
        }

        let resp_json: Value = resp.json().await?;
        let choice = &resp_json["choices"][0];
        let msg = &choice["message"];

        let content = msg["content"].as_str().unwrap_or("").to_string();
        let finish_reason = choice["finish_reason"].as_str().map(|s| s.to_string());

        let mut tool_calls = Vec::new();
        if let Some(calls) = msg["tool_calls"].as_array() {
            for c in calls {
                let id = c["id"].as_str().unwrap_or("").to_string();
                let name = c["function"]["name"].as_str().unwrap_or("").to_string();
                let args_str = c["function"]["arguments"].as_str().unwrap_or("{}");
                let arguments: Value = serde_json::from_str(args_str).unwrap_or(json!({}));
                tool_calls.push(ToolCall { id, name, arguments });
            }
        }

        Ok(LLMResponse {
            content,
            tool_calls,
            finish_reason,
        })
    }
}
