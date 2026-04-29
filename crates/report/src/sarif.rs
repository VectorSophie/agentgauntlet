use crate::AgentResults;
use anyhow::Result;
use serde_json::json;
use std::path::Path;

pub fn write_sarif(results: &[AgentResults], output_path: &Path) -> Result<()> {
    let mut runs = Vec::new();

    for agent_result in results {
        let mut sarif_results = Vec::new();

        for run in agent_result.runs {
            for finding in &run.findings {
                let level = match finding.severity {
                    agentgauntlet_core::types::Severity::Critical
                    | agentgauntlet_core::types::Severity::High => "error",
                    agentgauntlet_core::types::Severity::Medium => "warning",
                    agentgauntlet_core::types::Severity::Low => "note",
                };

                let step_id = run
                    .turns
                    .iter()
                    .find(|t| t.index == finding.turn_index)
                    .map(|t| t.step_id.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                sarif_results.push(json!({
                    "ruleId": finding.rule_id,
                    "level": level,
                    "message": {
                        "text": format!("{} | Agent: {}", finding.message, agent_result.display_name)
                    },
                    "locations": [{
                        "logicalLocations": [{
                            "name": format!("{}/{}", run.scenario_name, step_id)
                        }]
                    }]
                }));
            }
        }

        // Only add rules that are present in the results to satisfy SARIF minimum requirements
        let mut rules = std::collections::HashSet::new();
        for res in &sarif_results {
            if let Some(r_id) = res["ruleId"].as_str() {
                rules.insert(r_id.to_string());
            }
        }

        let rule_objects: Vec<_> = rules
            .into_iter()
            .map(|r| {
                json!({
                    "id": r,
                    "helpUri": "https://github.com/VectorSophie/agentgauntlet/tree/main/docs"
                })
            })
            .collect();

        runs.push(json!({
            "tool": {
                "driver": {
                    "name": "agentgauntlet",
                    "version": "0.1.0",
                    "informationUri": "https://github.com/VectorSophie/agentgauntlet",
                    "rules": rule_objects
                }
            },
            "results": sarif_results
        }));
    }

    let sarif_log = json!({
        "$schema": "https://docs.oasis-open.org/sarif/sarif/v2.1.0/os/schemas/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": runs
    });

    std::fs::write(output_path, serde_json::to_string_pretty(&sarif_log)?)?;
    Ok(())
}
