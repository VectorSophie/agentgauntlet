pub mod agent;
pub mod cli;
pub mod http;
pub mod ollama;
pub mod openai_compat;
pub mod stateless_cli;
pub mod mcp;

pub use agent::Agent;
pub use cli::{BuiltinAgent, BuiltinAgentFn, CliAgent};
pub use http::HttpAgent;
pub use ollama::OllamaAdapter;
pub use openai_compat::OpenAiCompatAdapter;
pub use stateless_cli::StatelessCliAdapter;
pub use mcp::McpAdapter;

use agentgauntlet_core::types::AgentResponse;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
impl Agent for CliAgent {
    async fn send_turn(
        &mut self,
        turn: usize,
        user: &str,
        timeout_ms: u64,
    ) -> Result<AgentResponse> {
        CliAgent::send_turn(self, turn, user, timeout_ms).await
    }
}

#[async_trait]
impl Agent for HttpAgent {
    async fn send_turn(
        &mut self,
        turn: usize,
        user: &str,
        timeout_ms: u64,
    ) -> Result<AgentResponse> {
        HttpAgent::send_turn(self, turn, user, timeout_ms).await
    }
}

#[async_trait]
impl Agent for BuiltinAgent {
    async fn send_turn(
        &mut self,
        turn: usize,
        user: &str,
        timeout_ms: u64,
    ) -> Result<AgentResponse> {
        BuiltinAgent::send_turn(self, turn, user, timeout_ms).await
    }
}
