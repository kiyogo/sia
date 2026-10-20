use crate::error::{CoreError, CoreResult};
use agent_tools::ToolDefinition;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cached_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub usage: ModelUsage,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn generate(&self, req: &ModelRequest) -> CoreResult<ModelResponse>;
    async fn test_connection(&self) -> CoreResult<bool>;
}

pub struct OpenAiCompatibleProvider {
    base_url: String,
    api_key: String,
    default_model: String,
    client: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(base_url: String, api_key: String, default_model: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            default_model,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl ModelProvider for OpenAiCompatibleProvider {
    fn name(&self) -> &str {
        "OpenAI-Compatible"
    }

    async fn test_connection(&self) -> CoreResult<bool> {
        let url = format!("{}/models", self.base_url);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if !self.api_key.is_empty() {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", self.api_key)) {
                headers.insert(AUTHORIZATION, val);
            }
        }

        let resp = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| CoreError::Model(format!("Failed to connect to provider at {}: {e}", self.base_url)))?;

        Ok(resp.status().is_success() || resp.status().as_u16() == 401)
    }

    async fn generate(&self, req: &ModelRequest) -> CoreResult<ModelResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if !self.api_key.is_empty() {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", self.api_key)) {
                headers.insert(AUTHORIZATION, val);
            }
        }

        let mut body = json!({
            "model": if req.model.is_empty() { &self.default_model } else { &req.model },
            "messages": req.messages,
            "temperature": req.temperature.unwrap_or(0.2),
        });

        if let Some(max_t) = req.max_tokens {
            body["max_tokens"] = json!(max_t);
        }

        if let Some(tools) = &req.tools {
            if !tools.is_empty() {
                let formatted_tools: Vec<Value> = tools
                    .iter()
                    .map(|t| {
                        json!({
                            "type": "function",
                            "function": {
                                "name": t.name,
                                "description": t.description,
                                "parameters": t.parameters
                            }
                        })
                    })
                    .collect();
                body["tools"] = json!(formatted_tools);
                body["tool_choice"] = json!("auto");
            }
        }

        debug!("Sending request to {}: {}", url, body);

        let res = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| CoreError::Model(format!("Request to provider failed: {e}")))?;

        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| CoreError::Model(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            return Err(CoreError::Model(format!(
                "Provider returned HTTP {}: {}",
                status, text
            )));
        }

        let parsed: Value = serde_json::from_str(&text)
            .map_err(|e| CoreError::Model(format!("Invalid JSON from provider: {e} | Text: {text}")))?;

        let choice = parsed
            .get("choices")
            .and_then(|c| c.get(0))
            .ok_or_else(|| CoreError::Model("Provider response missing 'choices'".to_string()))?;

        let message = choice
            .get("message")
            .ok_or_else(|| CoreError::Model("Choice missing 'message'".to_string()))?;

        let content = message
            .get("content")
            .and_then(|c| c.as_str())
            .map(String::from);

        let mut tool_calls = Vec::new();
        if let Some(calls) = message.get("tool_calls").and_then(|tc| tc.as_array()) {
            for call in calls {
                if let (Some(id), Some(fn_obj)) = (
                    call.get("id").and_then(|v| v.as_str()),
                    call.get("function"),
                ) {
                    let fn_name = fn_obj
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let fn_args = fn_obj
                        .get("arguments")
                        .and_then(|v| v.as_str())
                        .unwrap_or("{}")
                        .to_string();

                    tool_calls.push(ToolCall {
                        id: id.to_string(),
                        r#type: "function".to_string(),
                        function: FunctionCall {
                            name: fn_name,
                            arguments: fn_args,
                        },
                    });
                }
            }
        }

        let usage_obj = parsed.get("usage");
        let prompt_tokens = usage_obj
            .and_then(|u| u.get("prompt_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let completion_tokens = usage_obj
            .and_then(|u| u.get("completion_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let total_tokens = usage_obj
            .and_then(|u| u.get("total_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or((prompt_tokens + completion_tokens) as u64) as u32;

        let cached_tokens = usage_obj
            .and_then(|u| u.get("prompt_tokens_details"))
            .and_then(|d| d.get("cached_tokens"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        Ok(ModelResponse {
            content,
            tool_calls,
            usage: ModelUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
                cached_tokens,
            },
        })
    }
}

pub struct MockModelProvider {
    responses: tokio::sync::Mutex<Vec<ModelResponse>>,
}

impl MockModelProvider {
    pub fn new(responses: Vec<ModelResponse>) -> Self {
        Self {
            responses: tokio::sync::Mutex::new(responses),
        }
    }
}

#[async_trait]
impl ModelProvider for MockModelProvider {
    fn name(&self) -> &str {
        "Mock"
    }

    async fn test_connection(&self) -> CoreResult<bool> {
        Ok(true)
    }

    async fn generate(&self, _req: &ModelRequest) -> CoreResult<ModelResponse> {
        let mut list = self.responses.lock().await;
        if list.is_empty() {
            Ok(ModelResponse {
                content: Some("Task completed successfully.".to_string()),
                tool_calls: Vec::new(),
                usage: ModelUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                    cached_tokens: None,
                },
            })
        } else {
            Ok(list.remove(0))
        }
    }
}
