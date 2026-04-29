use crate::types::Run;
use anyhow::Result;
use std::path::Path;

pub fn load_run(run_dir: &Path) -> Result<Run> {
    let report_path = run_dir.join("report.json");
    let content = std::fs::read_to_string(&report_path)?;
    let run: Run = serde_json::from_str(&content)?;
    Ok(run)
}

pub fn list_runs(runs_dir: &Path) -> Result<Vec<String>> {
    if !runs_dir.exists() {
        return Ok(Vec::new());
    }
    let mut ids = Vec::new();
    for entry in std::fs::read_dir(runs_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                ids.push(name.to_string());
            }
        }
    }
    ids.sort();
    Ok(ids)
}
