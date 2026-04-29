use crate::agent::Agent;
use agentgauntlet_core::types::AgentResponse;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::process::Command;

/// How to extract the assistant's reply from raw process output.
enum OutputParser {
    /// Raw stdout is the reply.
    PlainText,
    /// opencode --format json: parse JSON event lines, find assistant content.
    OpenCodeJson,
}

/// Adapter for stateless CLI tools (one process per turn).
/// The full conversation history is injected into each prompt so the model
/// has context even though the process is fresh each time.
pub struct StatelessCliAdapter {
    binary: String,
    /// Args that go *before* the prompt argument.
    pre_args: Vec<String>,
    /// Args that go *after* the prompt argument.
    post_args: Vec<String>,
    history: Vec<(String, String)>,
    parser: OutputParser,
}

impl StatelessCliAdapter {
    /// opencode run --format json -m <model> "<prompt>"
    pub fn opencode(model: &str) -> Self {
        StatelessCliAdapter {
            binary: "opencode".to_string(),
            pre_args: vec![
                "run".to_string(),
                "--format".to_string(),
                "json".to_string(),
                "-m".to_string(),
                model.to_string(),
            ],
            post_args: vec![],
            history: vec![],
            parser: OutputParser::OpenCodeJson,
        }
    }

    /// claude -p "<prompt>"
    pub fn claude_code() -> Self {
        StatelessCliAdapter {
            binary: "claude".to_string(),
            pre_args: vec!["-p".to_string()],
            post_args: vec![],
            history: vec![],
            parser: OutputParser::PlainText,
        }
    }

    /// gemini "<prompt>"  (Gemini CLI)
    pub fn gemini() -> Self {
        StatelessCliAdapter {
            binary: "gemini".to_string(),
            pre_args: vec![],
            post_args: vec![],
            history: vec![],
            parser: OutputParser::PlainText,
        }
    }

    /// aider --message "<prompt>" --no-git
    pub fn aider() -> Self {
        StatelessCliAdapter {
            binary: "aider".to_string(),
            pre_args: vec!["--message".to_string()],
            post_args: vec!["--no-git".to_string(), "--yes".to_string()],
            history: vec![],
            parser: OutputParser::PlainText,
        }
    }

    fn build_prompt(&self, user: &str) -> String {
        if self.history.is_empty() {
            return user.to_string();
        }
        let mut parts = vec!["[Previous conversation:".to_string()];
        for (u, a) in &self.history {
            parts.push(format!("User: {u}"));
            parts.push(format!("Assistant: {a}"));
        }
        parts.push(String::from("]"));
        parts.push(String::new());
        parts.push(user.to_string());
        parts.join("\n")
    }

    fn parse_output(&self, raw: &str) -> String {
        match self.parser {
            OutputParser::PlainText => raw.trim().to_string(),
            OutputParser::OpenCodeJson => parse_opencode_json(raw),
        }
    }
}

#[async_trait]
impl Agent for StatelessCliAdapter {
    async fn send_turn(&mut self, _turn: usize, user: &str, timeout_ms: u64) -> Result<AgentResponse> {
        let prompt = self.build_prompt(user);
        let binary = self.binary.clone();
        let pre_args = self.pre_args.clone();
        let post_args = self.post_args.clone();

        let raw = tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new(&binary);
            cmd.args(&pre_args);
            cmd.arg(&prompt);
            cmd.args(&post_args);

            let out = cmd.output().context("Failed to spawn CLI agent")?;

            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();

            if stdout.trim().is_empty() && !stderr.trim().is_empty() {
                // Some CLIs write to stderr even on success
                Ok::<String, anyhow::Error>(stderr)
            } else {
                Ok::<String, anyhow::Error>(stdout)
            }
        })
        .await
        .context("Spawn-blocking panicked")??;

        // Apply timeout check after the fact by checking if we exceeded it.
        // For a proper timeout we'd need to use tokio::process::Command instead,
        // but spawn_blocking doesn't easily support cancellation.
        // Future improvement: migrate to tokio::process::Command.
        let _ = timeout_ms;

        if raw.trim().is_empty() {
            bail!("CLI agent produced no output");
        }

        let content = self.parse_output(&raw);
        self.history.push((user.to_string(), content.clone()));

        Ok(AgentResponse {
            output: content,
            tool_calls: vec![],
            memory_events: vec![],
        })
    }
}

/// Extract the assistant's reply from opencode's JSONL event stream.
fn parse_opencode_json(raw: &str) -> String {
    let mut parts: Vec<String> = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };

        // opencode emits various event types; extract text content from assistant events
        let event_type = val["type"].as_str().unwrap_or("");
        match event_type {
            "assistant" => {
                if let Some(text) = val["content"].as_str() {
                    parts.push(text.to_string());
                }
            }
            "text" | "content" => {
                if let Some(text) = val["text"].as_str().or_else(|| val["content"].as_str()) {
                    parts.push(text.to_string());
                }
            }
            "error" => {
                let msg = val["error"]["data"]["message"]
                    .as_str()
                    .or_else(|| val["error"]["message"].as_str())
                    .unwrap_or("unknown error");
                return format!("[ERROR] {msg}");
            }
            _ => {}
        }
    }

    if parts.is_empty() {
        // Fall back: return first non-JSON line as plain text
        raw.lines()
            .find(|l| !l.trim().is_empty() && !l.trim_start().starts_with('{'))
            .unwrap_or("[no response]")
            .to_string()
    } else {
        parts.join(" ")
    }
}
