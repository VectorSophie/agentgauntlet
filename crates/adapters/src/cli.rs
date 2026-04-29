use agentgauntlet_core::types::AgentResponse;
use anyhow::{bail, Context, Result};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use tracing::debug;

pub struct CliAgent {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    history: Vec<serde_json::Value>,
}

impl CliAgent {
    pub fn spawn(command: &str) -> Result<Self> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            bail!("Empty command");
        }
        let mut cmd = Command::new(parts[0]);
        cmd.args(&parts[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to spawn agent: {}", command))?;

        let stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;

        Ok(CliAgent {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            history: Vec::new(),
        })
    }

    pub async fn send_turn(
        &mut self,
        turn: usize,
        user: &str,
        timeout_ms: u64,
    ) -> Result<AgentResponse> {
        let msg = json!({
            "turn": turn,
            "user": user,
            "history": self.history
        });

        let line = serde_json::to_string(&msg)?;
        debug!("-> {}", line);
        writeln!(self.stdin, "{}", line)?;
        self.stdin.flush()?;

        let response = tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            self.read_response(),
        )
        .await
        .with_context(|| format!("Agent timed out after {}ms", timeout_ms))??;

        self.history.push(json!({
            "role": "user",
            "content": user
        }));
        self.history.push(json!({
            "role": "assistant",
            "content": response.output.clone()
        }));

        Ok(response)
    }

    async fn read_response(&mut self) -> Result<AgentResponse> {
        let mut line = String::new();
        loop {
            line.clear();
            let n = self.stdout.read_line(&mut line)?;
            if n == 0 {
                bail!("Agent process closed stdout unexpectedly");
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            debug!("<- {}", trimmed);
            match serde_json::from_str::<AgentResponse>(trimmed) {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    // Try to parse as a plain text fallback
                    tracing::warn!("Could not parse agent response as JSON: {e}. Line: {trimmed}");
                    return Ok(AgentResponse {
                        output: trimmed.to_string(),
                        tool_calls: vec![],
                        memory_events: vec![],
                    });
                }
            }
        }
    }

    pub fn kill(&mut self) {
        let _ = self.child.kill();
    }
}

impl Drop for CliAgent {
    fn drop(&mut self) {
        self.kill();
    }
}

// Builtin agent function type - for demo mode
pub type BuiltinAgentFn =
    Box<dyn Fn(usize, &str, &[serde_json::Value]) -> AgentResponse + Send + Sync>;

pub struct BuiltinAgent {
    handler: BuiltinAgentFn,
    history: Vec<serde_json::Value>,
}

impl BuiltinAgent {
    pub fn new(handler: BuiltinAgentFn) -> Self {
        BuiltinAgent {
            handler,
            history: Vec::new(),
        }
    }

    pub async fn send_turn(
        &mut self,
        turn: usize,
        user: &str,
        _timeout_ms: u64,
    ) -> Result<AgentResponse> {
        let response = (self.handler)(turn, user, &self.history);
        self.history.push(json!({"role": "user", "content": user}));
        self.history
            .push(json!({"role": "assistant", "content": response.output.clone()}));
        Ok(response)
    }
}
