use crate::Agent;
use agentgauntlet_core::types::{AgentResponse, AgentToolCall};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct McpAdapter {
    endpoint: String,
    history: Vec<serde_json::Value>,
    client: Client,
}

impl McpAdapter {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            history: Vec::new(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Agent for McpAdapter {
    async fn send_turn(
        &mut self,
        _turn: usize,
        user: &str,
        _timeout_ms: u64,
    ) -> Result<AgentResponse> {
        self.history.push(json!({
            "role": "user",
            "content": { "type": "text", "text": user }
        }));

        let req = json!({
            "jsonrpc": "2.0",
            "id": uuid::Uuid::new_v4().to_string(),
            "method": "sampling/createMessage",
            "params": {
                "messages": self.history,
                "maxTokens": 4096
            }
        });

        let resp = self.client.post(&self.endpoint).json(&req).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("MCP error: {} - {}", status, text));
        }

        let resp_json: serde_json::Value = resp.json().await?;
        let result = resp_json
            .get("result")
            .ok_or_else(|| anyhow!("No result in MCP response"))?;

        let mut output = String::new();
        let mut tool_calls = Vec::new();

        if let Some(content_arr) = result.get("content").and_then(|c| c.as_array()) {
            for item in content_arr {
                if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        output.push_str(text);
                    }
                } else if item.get("type").and_then(|t| t.as_str()) == Some("toolUseBlock") {
                    if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                        let args = item.get("parameters").cloned().unwrap_or(json!({}));
                        tool_calls.push(AgentToolCall {
                            name: name.to_string(),
                            args,
                        });
                    }
                }
            }

            self.history.push(json!({
                "role": "assistant",
                "content": content_arr.clone()
            }));
        } else {
            if let Some(role) = result.get("role") {
                self.history.push(json!({
                   "role": role,
                   "content": result.get("content").cloned().unwrap_or(json!([]))
                }));
            }
        }

        Ok(AgentResponse {
            output,
            tool_calls,
            memory_events: Vec::new(),
        })
    }
}
