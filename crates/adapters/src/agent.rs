use agentgauntlet_core::types::AgentResponse;
use anyhow::Result;
use async_trait::async_trait;

/// Common interface for all agent adapters.
#[async_trait]
pub trait Agent: Send {
    async fn send_turn(&mut self, turn: usize, user: &str, timeout_ms: u64) -> Result<AgentResponse>;
}
