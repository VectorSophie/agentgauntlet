use crate::agent::Agent;
use agentgauntlet_core::types::AgentResponse;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct OllamaAdapter {
    client: reqwest::Client,
    base_url: String,
    model: String,
    history: Vec<Value>,
}

impl OllamaAdapter {
    pub fn new(base_url: &str, model: &str) -> Self {
        OllamaAdapter {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            history: Vec::new(),
        }
    }
}

#[async_trait]
impl Agent for OllamaAdapter {
    async fn send_turn(&mut self, _turn: usize, user: &str, timeout_ms: u64) -> Result<AgentResponse> {
        self.history.push(json!({"role": "user", "content": user}));

        let payload = json!({
            "model": self.model,
            "messages": self.history,
            "stream": false,
        });

        let response = tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            self.client
                .post(format!("{}/api/chat", self.base_url))
                .json(&payload)
                .send(),
        )
        .await
        .with_context(|| format!("Ollama timed out after {timeout_ms}ms"))??;

        let body: Value = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        let content = body["message"]["content"]
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
