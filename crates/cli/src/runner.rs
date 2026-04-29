use agentgauntlet_adapters::{Agent, CliAgent, HttpAgent};
use agentgauntlet_core::types::{AgentResponse, Run};
use agentgauntlet_eval as eval;
use agentgauntlet_report as report;
use agentgauntlet_scenario::schema::{Scenario, TargetKind};
use anyhow::Result;
use std::path::Path;
use uuid::Uuid;

/// Run a scenario using the agent determined by `scenario.target`.
pub async fn run_scenario(scenario: &Scenario, runs_dir: &Path) -> Result<Run> {
    if scenario.target.kind == TargetKind::Builtin {
        let demo_runner = agentgauntlet_demo::DemoRunner::new(runs_dir.to_path_buf());
        return demo_runner.run_scenario(scenario).await;
    }

    let mut agent: Box<dyn Agent> = match &scenario.target.kind {
        TargetKind::Cli => {
            let command = scenario
                .target
                .command
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("CLI target requires 'command' field"))?;
            Box::new(CliAgent::spawn(command)?)
        }
        TargetKind::Http => {
            let url = scenario
                .target
                .url
                .clone()
                .ok_or_else(|| anyhow::anyhow!("HTTP target requires 'url' field"))?;
            Box::new(HttpAgent::new(url))
        }
        TargetKind::Builtin => unreachable!(),
        TargetKind::External => anyhow::bail!(
            "Scenario '{}' has target type 'external'. Use `agentgauntlet scan` or \
             call run_scenario_with_agent() directly.",
            scenario.name
        ),
    };

    run_with_agent(scenario, &mut *agent, runs_dir).await
}

/// Run a scenario with an already-constructed agent (used by `scan`).
pub async fn run_scenario_with_agent(
    scenario: &Scenario,
    agent: &mut dyn Agent,
    runs_dir: &Path,
) -> Result<Run> {
    run_with_agent(scenario, agent, runs_dir).await
}

async fn run_with_agent(
    scenario: &Scenario,
    agent: &mut dyn Agent,
    runs_dir: &Path,
) -> Result<Run> {
    let run_id = Uuid::new_v4().to_string();
    let run_dir = runs_dir.join(&run_id);
    std::fs::create_dir_all(&run_dir)?;

    std::fs::write(
        run_dir.join("scenario.yaml"),
        serde_yaml::to_string(scenario)?,
    )?;

    let mut run = Run::new(run_id.clone(), scenario.name.clone());

    report::write_trace_event(
        &run_dir,
        &serde_json::json!({"type": "run_started", "run_id": run_id, "scenario": scenario.name}),
    )?;

    for (i, step) in scenario.steps.iter().enumerate() {
        report::write_trace_event(
            &run_dir,
            &serde_json::json!({"type": "turn_started", "turn": i, "step_id": step.id}),
        )?;

        let response = match agent
            .send_turn(i + 1, &step.user, scenario.timeout_ms)
            .await
        {
            Ok(r) => r,
            Err(e) => AgentResponse {
                output: format!("[ERROR] {e}"),
                tool_calls: vec![],
                memory_events: vec![],
            },
        };

        report::write_trace_event(
            &run_dir,
            &serde_json::json!({"type": "user", "turn": i, "content": step.user}),
        )?;
        report::write_trace_event(
            &run_dir,
            &serde_json::json!({"type": "agent_output", "turn": i, "content": response.output}),
        )?;

        for tc in &response.tool_calls {
            report::write_trace_event(
                &run_dir,
                &serde_json::json!({"type": "tool_call", "turn": i, "name": tc.name, "args": tc.args}),
            )?;
        }
        for me in &response.memory_events {
            report::write_trace_event(
                &run_dir,
                &serde_json::json!({
                    "type": "memory_event", "turn": i,
                    "kind": me.kind, "key": me.key,
                    "value_preview": report::redact_if_sensitive(&me.value)
                }),
            )?;
        }

        let turn = eval::process_turn(i, step, &response, scenario);
        for finding in &turn.findings {
            report::write_trace_event(
                &run_dir,
                &serde_json::json!({
                    "type": "finding", "turn": i,
                    "severity": format!("{}", finding.severity),
                    "rule_id": finding.rule_id, "evidence": finding.evidence
                }),
            )?;
        }
        run.turns.push(turn);
    }

    let extra = eval::post_run_evaluation(&run.turns, scenario);
    if !extra.is_empty() {
        if let Some(last) = run.turns.last_mut() {
            last.findings.extend(extra);
        }
    }

    run.finalize();

    report::write_trace_event(
        &run_dir,
        &serde_json::json!({"type": "run_completed", "score": run.score.score}),
    )?;

    report::write_all(&run, scenario, &run_dir)?;

    Ok(run)
}
