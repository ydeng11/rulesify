use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const DEFAULT_MODEL: &str = "anthropic/claude-3.5-haiku";
const MAX_RETRIES: u32 = 3;

#[derive(Debug, Clone)]
pub struct OpenRouterClient {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: String,
}

impl OpenRouterClient {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .context("OPENROUTER_API_KEY environment variable not set")?;
        if api_key.trim().is_empty() {
            return Err(anyhow::anyhow!("OPENROUTER_API_KEY is empty - please set a valid API key"));
        }
        let model = std::env::var("OPENROUTER_MODEL")
            .ok()
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            api_key,
            model,
        })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub async fn classify_batch(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            max_tokens: 4096,
        };

        log::debug!(
            "Sending request to OpenRouter: model={}, max_tokens=4096, prompt_len={} bytes",
            self.model,
            user_prompt.len()
        );

        for attempt in 0..MAX_RETRIES {
            let response = self
                .client
                .post(OPENROUTER_API_URL)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .header("HTTP-Referer", "https://github.com/rulesify")
                .json(&request)
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    let chat_response: ChatResponse =
                        resp.json().await.context("Failed to parse LLM response")?;

                    if let Some(choice) = chat_response.choices.first() {
                        let content = choice.message.content.clone();
                        log::debug!("LLM returned {} bytes", content.len());
                        return Ok(content);
                    }
                    return Err(anyhow::anyhow!("No response from LLM"));
                }
                Ok(resp) if resp.status() == 429 => {
                    let wait_secs = 2u64.pow(attempt);
                    log::warn!("Rate limited, waiting {} seconds before retry", wait_secs);
                    tokio::time::sleep(Duration::from_secs(wait_secs)).await;
                    continue;
                }
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!("LLM API error ({}): {}", status, body));
                }
                Err(e) if attempt < MAX_RETRIES - 1 => {
                    log::warn!("Network error, retrying: {}", e);
                    continue;
                }
                Err(e) => {
                    return Err(e).context("Failed to call LLM API after retries");
                }
            }
        }

        Err(anyhow::anyhow!("Max retries exceeded"))
    }
}
