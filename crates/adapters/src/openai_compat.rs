use crate::agent::Agent;
use agentgauntlet_core::types::AgentResponse;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

/// Adapter for any OpenAI-compatible REST API (LM Studio, llama.cpp, etc.).
pub struct OpenAiCompatAdapter {
    client: reqwest::Client,
    base_url: String,
    model: String,
    api_key: String,
    history: Vec<Value>,
}

impl OpenAiCompatAdapter {
    pub fn new(base_url: &str, model: &str, api_key: &str) -> Self {
        OpenAiCompatAdapter {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            api_key: api_key.to_string(),
            history: Vec::new(),
        }
    }

    pub fn lmstudio(base_url: &str, model: &str) -> Self {
        Self::new(base_url, model, "lm-studio")
    }
}

#[async_trait]
impl Agent for OpenAiCompatAdapter {
    async fn send_turn(&mut self, _turn: usize, user: &str, timeout_ms: u64) -> Result<AgentResponse> {
        self.history.push(json!({"role": "user", "content": user}));

        let payload = json!({
            "model": self.model,
            "messages": self.history,
            "max_tokens": 1024,
            "temperature": 0.1,
            "stream": false,
        });

        let response = tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            self.client
                .post(format!("{}/v1/chat/completions", self.base_url))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&payload)
                .send(),
        )
        .await
        .with_context(|| format!("OpenAI-compat API timed out after {timeout_ms}ms"))??;

        let body: Value = response
            .json()
            .await
            .context("Failed to parse OpenAI-compat response")?;

        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("[no response]")
            .to_string();

        self.history.push(json!({"role": "assistant", "content": content}));

        Ok(AgentResponse {
            output: content,
            tool_calls: vec![],
            memory_events: vec![],
        })
    }
}
