use serde_json::Value;
use std::process::Command;
use std::time::Duration;

/// A detected AI agent on the local system.
#[derive(Debug, Clone)]
pub enum DetectedAgent {
    /// Ollama server with a specific loaded model.
    Ollama { base_url: String, model: String },
    /// LM Studio (or any OpenAI-compatible server) with a specific model.
    LmStudio { base_url: String, model: String },
    /// OpenCode CLI (stateless per invocation).
    OpenCode { version: String },
    /// Claude Code CLI (stateless per invocation).
    ClaudeCode { version: String },
    /// Gemini CLI (stateless per invocation).
    GeminiCli { version: String },
    /// Aider CLI (stateless per invocation).
    Aider { version: String },
    /// MCP Agent via JSON-RPC
    Mcp { endpoint: String },
}

impl DetectedAgent {
    /// Human-readable label, e.g. "Ollama / llama3.2:latest".
    pub fn display_name(&self) -> String {
        match self {
            DetectedAgent::Ollama { model, .. } => format!("Ollama / {model}"),
            DetectedAgent::LmStudio { model, .. } => format!("LM Studio / {model}"),
            DetectedAgent::OpenCode { .. } => "OpenCode".to_string(),
            DetectedAgent::ClaudeCode { .. } => "Claude Code".to_string(),
            DetectedAgent::GeminiCli { .. } => "Gemini CLI".to_string(),
            DetectedAgent::Aider { .. } => "Aider".to_string(),
            DetectedAgent::Mcp { endpoint } => format!("MCP / {endpoint}"),
        }
    }

    /// Filesystem-safe ID for report filenames, e.g. "ollama_llama3.2".
    pub fn file_id(&self) -> String {
        match self {
            DetectedAgent::Ollama { model, .. } => {
                format!("ollama_{}", sanitize(model))
            }
            DetectedAgent::LmStudio { model, .. } => {
                format!("lmstudio_{}", sanitize(model))
            }
            DetectedAgent::OpenCode { .. } => "opencode".to_string(),
            DetectedAgent::ClaudeCode { .. } => "claude_code".to_string(),
            DetectedAgent::GeminiCli { .. } => "gemini_cli".to_string(),
            DetectedAgent::Aider { .. } => "aider".to_string(),
            DetectedAgent::Mcp { endpoint } => format!("mcp_{}", sanitize(endpoint)),
        }
    }

    /// Provider family, for grouping in the comparison report.
    pub fn provider(&self) -> &'static str {
        match self {
            DetectedAgent::Ollama { .. } => "Ollama",
            DetectedAgent::LmStudio { .. } => "LM Studio",
            DetectedAgent::OpenCode { .. } => "OpenCode",
            DetectedAgent::ClaudeCode { .. } => "Claude Code",
            DetectedAgent::GeminiCli { .. } => "Gemini CLI",
            DetectedAgent::Aider { .. } => "Aider",
            DetectedAgent::Mcp { .. } => "MCP",
        }
    }
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Detect all available AI agents. Runs HTTP probes and binary checks concurrently.
pub async fn detect_all() -> Vec<DetectedAgent> {
    let (ollama, lmstudio) = tokio::join!(
        detect_ollama("http://localhost:11434"),
        detect_lmstudio("http://localhost:1234"),
    );

    let mcp_endpoint = std::env::var("AGENTGAUNTLET_MCP_ENDPOINT").unwrap_or_else(|_| "http://localhost:3000/mcp".to_string());
    let mcp = detect_mcp(&mcp_endpoint).await;

    let (opencode, claude, gemini, aider) = tokio::join!(
        detect_binary("opencode", &["--version"]),
        detect_binary("claude", &["--version"]),
        detect_binary("gemini", &["--version"]),
        detect_binary("aider", &["--version"]),
    );

    let mut agents: Vec<DetectedAgent> = Vec::new();

    agents.extend(ollama);
    agents.extend(lmstudio);
    agents.extend(mcp);

    if let Some(v) = opencode {
        agents.push(DetectedAgent::OpenCode { version: v });
    }
    if let Some(v) = claude {
        agents.push(DetectedAgent::ClaudeCode { version: v });
    }
    if let Some(v) = gemini {
        agents.push(DetectedAgent::GeminiCli { version: v });
    }
    if let Some(v) = aider {
        agents.push(DetectedAgent::Aider { version: v });
    }

    agents
}

async fn detect_ollama(base_url: &str) -> Vec<DetectedAgent> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap_or_default();

    let url = format!("{base_url}/api/tags");
    let Ok(resp) = client.get(&url).send().await else {
        return vec![];
    };
    let Ok(body) = resp.json::<Value>().await else {
        return vec![];
    };

    let models = body["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["name"].as_str().map(String::from))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    models
        .into_iter()
        .map(|model| DetectedAgent::Ollama {
            base_url: base_url.to_string(),
            model,
        })
        .collect()
}

async fn detect_mcp(endpoint: &str) -> Vec<DetectedAgent> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap_or_default();

    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "probe",
        "method": "ping"
    });

    let Ok(resp) = client.post(endpoint).json(&req).send().await else {
        return vec![];
    };
    
    if resp.status().is_success() || resp.status().as_u16() == 404 || resp.status().as_u16() == 400 {
        // Since it responded to an HTTP POST at least, we'll tentatively count it
        vec![DetectedAgent::Mcp { endpoint: endpoint.to_string() }]
    } else {
        vec![]
    }
}

async fn detect_lmstudio(base_url: &str) -> Vec<DetectedAgent> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap_or_default();

    let url = format!("{base_url}/v1/models");
    let Ok(resp) = client.get(&url).send().await else {
        return vec![];
    };
    let Ok(body) = resp.json::<Value>().await else {
        return vec![];
    };

    let models = body["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["id"].as_str())
                // skip embedding-only models
                .filter(|id| !id.contains("embed") && !id.contains("embedding"))
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    models
        .into_iter()
        .map(|model| DetectedAgent::LmStudio {
            base_url: base_url.to_string(),
            model,
        })
        .collect()
}

/// Try running `binary args...`, return trimmed first line of stdout on success.
async fn detect_binary(binary: &str, args: &[&str]) -> Option<String> {
    let binary = binary.to_string();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    tokio::task::spawn_blocking(move || {
        let output = Command::new(&binary).args(&args).output().ok()?;

        if output.status.success() || !output.stdout.is_empty() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            // Return first non-empty line as version string
            Some(
                text.lines()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or("unknown")
                    .to_string(),
            )
        } else {
            None
        }
    })
    .await
    .ok()
    .flatten()
}

/// Result of a single detection probe for display purposes.
pub struct ProbeResult {
    pub label: String,
    pub found: bool,
    pub detail: String,
}

/// Run all probes and return display-ready results (for the scan header).
pub async fn probe_all() -> Vec<ProbeResult> {
    let agents = detect_all().await;

    // Group by provider for display
    let mut results: Vec<ProbeResult> = Vec::new();

    let ollama_found: Vec<_> = agents
        .iter()
        .filter(|a| matches!(a, DetectedAgent::Ollama { .. }))
        .collect();
    let lmstudio_found: Vec<_> = agents
        .iter()
        .filter(|a| matches!(a, DetectedAgent::LmStudio { .. }))
        .collect();

    if ollama_found.is_empty() {
        results.push(ProbeResult {
            label: "Ollama".to_string(),
            found: false,
            detail: "not found".to_string(),
        });
    } else {
        let models: Vec<&str> = ollama_found
            .iter()
            .filter_map(|a| {
                if let DetectedAgent::Ollama { model, .. } = a {
                    Some(model.as_str())
                } else {
                    None
                }
            })
            .collect();
        results.push(ProbeResult {
            label: "Ollama".to_string(),
            found: true,
            detail: models.join(", "),
        });
    }

    if lmstudio_found.is_empty() {
        results.push(ProbeResult {
            label: "LM Studio".to_string(),
            found: false,
            detail: "not found".to_string(),
        });
    } else {
        let models: Vec<&str> = lmstudio_found
            .iter()
            .filter_map(|a| {
                if let DetectedAgent::LmStudio { model, .. } = a {
                    Some(model.as_str())
                } else {
                    None
                }
            })
            .collect();
        results.push(ProbeResult {
            label: "LM Studio".to_string(),
            found: true,
            detail: models.join(", "),
        });
    }

    for (label, variant_name) in &[
        ("OpenCode", "OpenCode"),
        ("Claude Code", "ClaudeCode"),
        ("Gemini CLI", "GeminiCli"),
        ("Aider", "Aider"),
    ] {
        let found = agents.iter().find(|a| a.provider() == *label);
        if let Some(agent) = found {
            let version = match agent {
                DetectedAgent::OpenCode { version }
                | DetectedAgent::ClaudeCode { version }
                | DetectedAgent::GeminiCli { version }
                | DetectedAgent::Aider { version } => version.clone(),
                DetectedAgent::Mcp { endpoint } => endpoint.clone(),
                _ => String::new(),
            };
            results.push(ProbeResult {
                label: label.to_string(),
                found: true,
                detail: version,
            });
        } else {
            let _ = variant_name;
            results.push(ProbeResult {
                label: label.to_string(),
                found: false,
                detail: "not found".to_string(),
            });
        }
    }

    results
}
