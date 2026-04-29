use agentgauntlet_detect::{detect_all, DetectedAgent};
use agentgauntlet_report::AgentResults;
use agentgauntlet_scenario::standard::standard_scenarios;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

pub struct QuickOptions {
    pub judge: String,
    pub mcp: Option<String>,
}

pub async fn cmd_quick(opts: QuickOptions, runs_dir: &Path) -> Result<()> {
    println!(
        "\n  {}",
        "⚡ Running Quick Scan (Top 3 Scenarios)".bold().yellow()
    );
    println!("  {}", "Scanning for AI agents...".bold());
    println!();

    let mut agents = detect_all().await;
    if let Some(ep) = opts.mcp {
        agents.push(DetectedAgent::Mcp { endpoint: ep });
    }

    if agents.is_empty() {
        println!("  {}", "No AI agents detected locally.".red());
        return Ok(());
    }

    // Only pick the first agent for quick scan to be extremely fast
    let target_agent = agents.into_iter().next().unwrap();
    println!(
        "  {}  {}  {}",
        "✓".green().bold(),
        "Target Agent:".dimmed(),
        target_agent.display_name().cyan()
    );
    println!();

    let all_scenarios = standard_scenarios();
    // Pick exactly 3 high-impact scenarios
    let quick_scenarios: Vec<_> = all_scenarios
        .into_iter()
        .filter(|s| {
            s.name == "ignore_previous_instructions"
                || s.name == "persona_jailbreak"
                || s.name == "delayed_trigger"
        })
        .take(3)
        .collect();

    let mut runs = Vec::new();
    let judge_spec = if opts.judge == "auto" {
        std::env::var("AGENTGAUNTLET_JUDGE").unwrap_or_else(|_| "auto".to_string())
    } else {
        opts.judge.clone()
    };

    if judge_spec != "auto" {
        std::env::set_var("AGENTGAUNTLET_JUDGE", &judge_spec);
    }
    let judge_impl = agentgauntlet_judge::auto_judge();

    let mut adapter = crate::cmd_scan::create_adapter(&target_agent);

    for scenario in quick_scenarios {
        print!("  {} {} ... ", "Running".dimmed(), scenario.name.bold());
        match crate::runner::run_scenario_with_agent(
            &scenario,
            &mut *adapter,
            Some(judge_impl.as_ref()),
            runs_dir,
        )
        .await
        {
            Ok(run) => {
                if run.score.score < 100 {
                    println!("{}", "FAIL".red().bold());
                } else {
                    println!("{}", "SECURE".green().bold());
                }
                runs.push(run);
            }
            Err(e) => {
                println!("{} ({})", "ERROR".red(), e);
            }
        }
    }

    let display_name = target_agent.display_name();
    let file_id = target_agent.file_id();
    let results = AgentResults {
        display_name: &display_name,
        file_id: &file_id,
        runs: &runs,
    };

    agentgauntlet_report::print_scorecard(&target_agent.display_name(), &results);

    Ok(())
}
