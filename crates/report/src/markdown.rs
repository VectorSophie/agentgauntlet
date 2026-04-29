use agentgauntlet_core::types::{Run, Severity};
use agentgauntlet_scenario::schema::Scenario;
use anyhow::Result;
use std::path::Path;

pub fn write_transcript(run: &Run, scenario: &Scenario, dir: &Path) -> Result<()> {
    let path = dir.join("transcript.md");
    let content = build_markdown(run, scenario);
    std::fs::write(&path, content)?;
    Ok(())
}

fn build_markdown(run: &Run, scenario: &Scenario) -> String {
    let mut md = String::new();

    md.push_str(&format!(
        "# AgentGauntlet Report: {}\n\n",
        run.scenario_name
    ));
    md.push_str(&format!("**Category:** {}  \n", scenario.category));
    md.push_str(&format!("**Severity:** {}  \n", scenario.severity));
    md.push_str(&format!("**Run ID:** `{}`  \n\n", run.id));

    md.push_str(&format!(
        "## Score: {}/100 — {}\n\n",
        run.score.score, run.score.label
    ));

    md.push_str("## Findings Summary\n\n");
    md.push_str("| Severity | Count |\n");
    md.push_str("|----------|-------|\n");
    md.push_str(&format!("| Critical | {} |\n", run.score.critical));
    md.push_str(&format!("| High     | {} |\n", run.score.high));
    md.push_str(&format!("| Medium   | {} |\n", run.score.medium));
    md.push_str(&format!("| Low      | {} |\n", run.score.low));
    md.push('\n');

    if !run.findings.is_empty() {
        md.push_str("## Findings\n\n");
        for finding in &run.findings {
            let sev = match finding.severity {
                Severity::Critical => "CRITICAL",
                Severity::High => "HIGH",
                Severity::Medium => "MEDIUM",
                Severity::Low => "LOW",
            };
            md.push_str(&format!(
                "### [{sev}] turn_{} — `{}`\n\n",
                finding.turn_index + 1,
                finding.rule_id
            ));
            md.push_str(&format!("{}\n\n", finding.message));
            if !finding.evidence.is_empty() {
                md.push_str(&format!("**Evidence:** `{}`\n\n", finding.evidence));
            }
        }
    }

    md.push_str("## Conversation Transcript\n\n");
    for turn in &run.turns {
        md.push_str(&format!(
            "### Turn {} — `{}`\n\n",
            turn.index + 1,
            turn.step_id
        ));
        md.push_str(&format!("**User:** {}\n\n", turn.user_input));
        md.push_str(&format!("**Agent:** {}\n\n", turn.agent_output));

        if !turn.tool_calls.is_empty() {
            md.push_str("**Tool Calls:**\n\n");
            for tc in &turn.tool_calls {
                let status = if tc.allowed { "allowed" } else { "FORBIDDEN" };
                md.push_str(&format!(
                    "- `{}` ({status}) args: `{}`\n",
                    tc.name, tc.args_json
                ));
            }
            md.push('\n');
        }

        if !turn.memory_events.is_empty() {
            md.push_str("**Memory Events:**\n\n");
            for me in &turn.memory_events {
                md.push_str(&format!(
                    "- [{}] key={:?} risk={}\n",
                    me.kind, me.key, me.risk
                ));
            }
            md.push('\n');
        }

        if !turn.findings.is_empty() {
            md.push_str("**Findings:**\n\n");
            for f in &turn.findings {
                md.push_str(&format!(
                    "- [{}] {}: {}\n",
                    f.severity, f.rule_id, f.message
                ));
            }
            md.push('\n');
        }
    }

    md.push_str("## Reproduction\n\n");
    md.push_str("```bash\n");
    md.push_str(&format!("agentgauntlet replay {}\n", run.id));
    md.push_str("```\n");

    md
}
