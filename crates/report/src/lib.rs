pub mod agent_report;
pub mod comparison;
pub mod console;
pub mod json;
pub mod markdown;
pub mod sarif;

pub use agent_report::{write_agent_report, AgentResults};
pub use sarif::write_sarif;
pub use comparison::{write_comparison, AgentSummary};

use agentgauntlet_core::types::Run;
use agentgauntlet_scenario::schema::Scenario;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn write_trace_event(dir: &Path, event: &serde_json::Value) -> Result<()> {
    let path = dir.join("trace.jsonl");
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    serde_json::to_writer(&mut file, event)?;
    writeln!(file)?;
    Ok(())
}

pub fn write_all(run: &Run, scenario: &Scenario, dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    json::write_report(run, dir)?;
    markdown::write_transcript(run, scenario, dir)?;
    Ok(())
}

pub fn redact_if_sensitive(value: &str) -> String {
    let sensitive_patterns = &[r"sk-[A-Za-z0-9]{20,}", r"AKIA[0-9A-Z]{16}"];
    for pattern in sensitive_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if re.is_match(value) {
                let mut hasher = Sha256::new();
                hasher.update(value.as_bytes());
                let hash = hex::encode(hasher.finalize());
                return format!("[REDACTED sha256={}]", &hash[..16]);
            }
        }
    }
    value.chars().take(200).collect()
}
