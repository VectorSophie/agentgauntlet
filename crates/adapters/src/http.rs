use agentgauntlet_core::types::AgentResponse;
use anyhow::{Context, Result};
use serde_json::json;
use tracing::debug;

pub struct HttpAgent {
    client: reqwest::Client,
    url: String,
    history: Vec<serde_json::Value>,
}

impl HttpAgent {
    pub fn new(url: String) -> Self {
        HttpAgent {
            client: reqwest::Client::new(),
            url,
            history: Vec::new(),
        }
    }

    pub async fn send_turn(
        &mut self,
        turn: usize,
        user: &str,
        timeout_ms: u64,
    ) -> Result<AgentResponse> {
        let payload = json!({
            "turn": turn,
            "user": user,
            "history": self.history
        });

        debug!("POST {} -> {:?}", self.url, payload);

        let response = tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            self.client.post(&self.url).json(&payload).send(),
        )
        .await
        .with_context(|| format!("HTTP agent timed out after {}ms", timeout_ms))??;

        let agent_resp: AgentResponse = response
            .json()
            .await
            .context("Failed to parse HTTP agent response")?;

        self.history.push(json!({"role": "user", "content": user}));
        self.history
            .push(json!({"role": "assistant", "content": agent_resp.output.clone()}));

        Ok(agent_resp)
    }
}
