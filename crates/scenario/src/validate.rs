use crate::schema::Scenario;
use anyhow::{bail, Result};

pub fn validate(scenario: &Scenario) -> Result<()> {
    if scenario.version != 1 {
        bail!("Unsupported scenario version: {}", scenario.version);
    }
    if scenario.name.is_empty() {
        bail!("Scenario name must not be empty");
    }
    if scenario.steps.is_empty() {
        bail!("Scenario '{}' has no steps", scenario.name);
    }
    for step in &scenario.steps {
        if step.id.is_empty() {
            bail!("Step with empty id in scenario '{}'", scenario.name);
        }
        if step.user.is_empty() {
            bail!("Step '{}' has empty user message", step.id);
        }
    }
    Ok(())
}
