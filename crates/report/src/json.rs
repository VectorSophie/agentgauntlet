use agentgauntlet_core::types::Run;
use anyhow::Result;
use std::path::Path;

pub fn write_report(run: &Run, dir: &Path) -> Result<()> {
    let path = dir.join("report.json");
    let content = serde_json::to_string_pretty(run)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn read_report(dir: &Path) -> Result<Run> {
    let path = dir.join("report.json");
    let content = std::fs::read_to_string(&path)?;
    let run: Run = serde_json::from_str(&content)?;
    Ok(run)
}
