use crate::schema::Scenario;
use anyhow::{Context, Result};
use std::path::Path;

pub fn load_scenario(path: &Path) -> Result<Scenario> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read scenario file: {}", path.display()))?;
    let scenario: Scenario = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse scenario YAML: {}", path.display()))?;
    Ok(scenario)
}

pub fn find_scenarios(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut paths = Vec::new();
    find_recursive(dir, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn find_recursive(dir: &Path, paths: &mut Vec<std::path::PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            find_recursive(&path, paths)?;
        } else if let Some(ext) = path.extension() {
            if ext == "yaml" || ext == "yml" {
                paths.push(path);
            }
        }
    }
    Ok(())
}
