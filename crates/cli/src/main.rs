use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

mod cmd_quick;
mod cmd_scan;
mod runner;

#[derive(Parser)]
#[command(
    name = "agentgauntlet",
    about = "Security test runner for AI coding agents — auto-detects Ollama, LM Studio, Claude Code, OpenCode and more.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Auto-detect local AI agents, select which to test, run in parallel
    Scan {
        /// Path to a directory of scenario YAML files (default: built-in standard suite)
        #[arg(long)]
        dir: Option<PathBuf>,

        /// Skip interactive selection and test every detected agent
        #[arg(long)]
        all: bool,

        /// Max number of agents to test in parallel (default: all)
        #[arg(long)]
        parallel: Option<usize>,

        /// Directory to write AGENTGAUNTLET_*.md reports (default: current dir)
        #[arg(long, default_value = ".")]
        output: PathBuf,

        /// Judge model [auto|none|claude|ollama:model|lmstudio:model] (default: auto)
        #[arg(long, default_value = "auto")]
        judge: String,

        /// MCP endpoint to scan
        #[arg(long)]
        mcp: Option<String>,

        /// Output format [md|sarif|html]
        #[arg(long, default_value = "md")]
        format: String,
    },

    /// Initialize AgentGauntlet in the current directory
    Init,

    /// Quick scan (top 3 scenarios, 15 seconds)
    Quick {
        /// Judge model
        #[arg(long, default_value = "auto")]
        judge: String,

        /// MCP endpoint to scan
        #[arg(long)]
        mcp: Option<String>,
    },

    /// Run built-in demo scenarios against the vulnerable agent
    Demo,

    /// Run a single scenario file
    #[command(name = "scenario")]
    Scenario {
        #[command(subcommand)]
        action: ScenarioAction,
    },

    /// Run all scenarios in a directory
    Test {
        #[arg(long)]
        dir: Option<PathBuf>,

        #[arg(long)]
        ci: bool,

        #[arg(long, default_value = "high")]
        fail_on: String,
    },

    /// Replay a previous run's trace
    Replay { run_id: String },

    /// Show report for a run
    #[command(name = "report")]
    Report {
        #[command(subcommand)]
        action: ReportAction,
    },

    /// Print version information
    Version,
}

#[derive(Subcommand)]
enum ScenarioAction {
    Run { scenario_file: PathBuf },
}

#[derive(Subcommand)]
enum ReportAction {
    Show { run_id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let cli = Cli::parse();
    let config = load_config();
    let runs_dir = config.output_dir.clone();

    match cli.command {
        Commands::Scan {
            dir,
            all,
            parallel,
            output,
            judge,
            mcp,
            format,
        } => {
            cmd_scan::cmd_scan(
                cmd_scan::ScanOptions {
                    scenario_dir: dir,
                    select_all: all,
                    parallel,
                    output,
                    judge,
                    mcp,
                    format,
                },
                &runs_dir,
            )
            .await?;
        }
        Commands::Quick { judge, mcp } => {
            cmd_quick::cmd_quick(cmd_quick::QuickOptions { judge, mcp }, &runs_dir).await?;
        }
        Commands::Init => {
            cmd_init()?;
        }
        Commands::Demo => cmd_demo(&runs_dir).await?,
        Commands::Scenario {
            action: ScenarioAction::Run { scenario_file },
        } => {
            cmd_scenario_run(&scenario_file, &runs_dir).await?;
        }
        Commands::Test { dir, ci, fail_on } => {
            let scenarios_dir = dir.unwrap_or_else(|| PathBuf::from("scenarios/demo"));
            cmd_test(&scenarios_dir, &runs_dir, ci, &fail_on).await?;
        }
        Commands::Replay { run_id } => cmd_replay(&runs_dir, &run_id)?,
        Commands::Report {
            action: ReportAction::Show { run_id },
        } => {
            cmd_report_show(&runs_dir, &run_id)?;
        }
        Commands::Version => {
            println!("agentgauntlet {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}

struct Config {
    output_dir: PathBuf,
    #[allow(dead_code)]
    default_timeout_ms: u64,
}

fn load_config() -> Config {
    let config_path = PathBuf::from(".agentgauntlet/config.yaml");
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(val) = serde_yaml::from_str::<serde_json::Value>(&content) {
                let output_dir = val["output_dir"]
                    .as_str()
                    .unwrap_or(".agentgauntlet/runs")
                    .to_string();
                let timeout_ms = val["default_timeout_ms"].as_u64().unwrap_or(30000);
                return Config {
                    output_dir: PathBuf::from(output_dir),
                    default_timeout_ms: timeout_ms,
                };
            }
        }
    }
    Config {
        output_dir: PathBuf::from(".agentgauntlet/runs"),
        default_timeout_ms: 30000,
    }
}

fn cmd_init() -> Result<()> {
    for dir in &[".agentgauntlet", ".agentgauntlet/runs"] {
        std::fs::create_dir_all(dir)?;
    }
    let config_path = PathBuf::from(".agentgauntlet/config.yaml");
    if !config_path.exists() {
        std::fs::write(
            &config_path,
            "version: 1\noutput_dir: \".agentgauntlet/runs\"\ndefault_timeout_ms: 30000\nfail_on: \"critical\"\n",
        )?;
        println!("Initialized .agentgauntlet/");
    } else {
        println!(".agentgauntlet/ already exists");
    }
    Ok(())
}

async fn cmd_demo(runs_dir: &Path) -> Result<()> {
    println!("AgentGauntlet Demo");
    println!("==================");
    println!("Running vulnerable local agent...");
    println!();

    std::fs::create_dir_all(runs_dir)?;

    let demo_scenarios = agentgauntlet_demo::get_demo_scenarios();
    let runner = agentgauntlet_demo::DemoRunner::new(runs_dir.to_path_buf());
    let mut all_runs = Vec::new();

    for scenario in &demo_scenarios {
        print!("  Running: {} ... ", scenario.name);
        let run = runner.run_scenario(scenario).await?;
        let status = if run.score.critical > 0 || run.score.high > 0 {
            "[FAIL]"
        } else if run.score.medium > 0 {
            "[WARN]"
        } else {
            "[PASS]"
        };
        println!("{status}");
        println!("    {status} {}", scenario.name);
        all_runs.push(run);
    }

    println!();

    let total_findings: usize = all_runs.iter().map(|r| r.findings.len()).sum();
    let avg_score = if all_runs.is_empty() {
        100u32
    } else {
        all_runs.iter().map(|r| r.score.score as u32).sum::<u32>() / all_runs.len() as u32
    };

    println!("Security Score:");
    println!("  Average Score: {avg_score}/100");
    println!("  Total Findings: {total_findings}");
    println!(
        "  Critical: {}",
        all_runs.iter().map(|r| r.score.critical).sum::<usize>()
    );
    println!(
        "  High:     {}",
        all_runs.iter().map(|r| r.score.high).sum::<usize>()
    );
    println!(
        "  Medium:   {}",
        all_runs.iter().map(|r| r.score.medium).sum::<usize>()
    );
    println!(
        "  Low:      {}",
        all_runs.iter().map(|r| r.score.low).sum::<usize>()
    );
    println!();
    println!("Report written to: {}", runs_dir.display());
    println!();

    Ok(())
}

async fn cmd_scenario_run(scenario_file: &Path, runs_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(runs_dir)?;

    let scenario = agentgauntlet_scenario::load_scenario(scenario_file)
        .with_context(|| format!("Failed to load scenario: {}", scenario_file.display()))?;

    agentgauntlet_scenario::validate(&scenario)?;

    println!("Running scenario: {}", scenario.name);
    println!("Category: {}", scenario.category);
    println!("Severity: {}", scenario.severity);
    println!();

    let run = runner::run_scenario(&scenario, None, runs_dir).await?;
    let run_dir = runs_dir.join(&run.id);
    agentgauntlet_report::console::print_report(&run, &run_dir);

    Ok(())
}

async fn cmd_test(scenarios_dir: &Path, runs_dir: &Path, ci: bool, fail_on: &str) -> Result<()> {
    std::fs::create_dir_all(runs_dir)?;

    let scenario_files = agentgauntlet_scenario::find_scenarios(scenarios_dir)?;

    if scenario_files.is_empty() {
        println!("No scenario files found in: {}", scenarios_dir.display());
        return Ok(());
    }

    println!(
        "AgentGauntlet Test Run — {} scenarios",
        scenario_files.len()
    );
    println!("{}", "=".repeat(50));
    println!();

    let mut all_runs = Vec::new();

    for path in &scenario_files {
        let scenario = match agentgauntlet_scenario::load_scenario(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[ERROR] Failed to load {}: {}", path.display(), e);
                continue;
            }
        };
        if let Err(e) = agentgauntlet_scenario::validate(&scenario) {
            eprintln!("[ERROR] Invalid scenario {}: {}", path.display(), e);
            continue;
        }
        let run = runner::run_scenario(&scenario, None, runs_dir).await?;
        agentgauntlet_report::console::print_summary_line(&scenario.name, &run);
        all_runs.push(run);
    }

    println!();

    let total = all_runs.len();
    let failed = all_runs
        .iter()
        .filter(|r| r.score.critical > 0 || r.score.high > 0)
        .count();

    println!(
        "Results: {}/{} passed, {} failed",
        total - failed,
        total,
        failed
    );

    if ci {
        let should_fail = all_runs.iter().any(|r| match fail_on {
            "low" => !r.findings.is_empty(),
            "medium" => r.score.medium > 0 || r.score.high > 0 || r.score.critical > 0,
            "high" => r.score.high > 0 || r.score.critical > 0,
            "critical" => r.score.critical > 0,
            _ => r.score.high > 0 || r.score.critical > 0,
        });
        if should_fail {
            bail!("CI failure threshold reached (fail-on: {fail_on})");
        }
    }

    Ok(())
}

fn cmd_replay(runs_dir: &Path, run_id: &str) -> Result<()> {
    let trace_path = runs_dir.join(run_id).join("trace.jsonl");
    if !trace_path.exists() {
        bail!("Run not found: {}", run_id);
    }

    println!("Replay: {run_id}");
    println!("{}", "=".repeat(50));
    println!();

    let content = std::fs::read_to_string(&trace_path)?;
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        match val["type"].as_str().unwrap_or("") {
            "user" => println!(
                "[Turn {}] User: {}",
                val["turn"].as_u64().unwrap_or(0) + 1,
                val["content"].as_str().unwrap_or("")
            ),
            "agent_output" => println!(
                "[Turn {}] Agent: {}",
                val["turn"].as_u64().unwrap_or(0) + 1,
                val["content"].as_str().unwrap_or("")
            ),
            "tool_call" => println!(
                "[Turn {}] Tool: {}({})",
                val["turn"].as_u64().unwrap_or(0) + 1,
                val["name"].as_str().unwrap_or(""),
                val["args"]
            ),
            "finding" => println!(
                "[Turn {}] FINDING [{}]: {}",
                val["turn"].as_u64().unwrap_or(0) + 1,
                val["severity"].as_str().unwrap_or(""),
                val["rule_id"].as_str().unwrap_or("")
            ),
            "run_completed" => {
                println!();
                println!(
                    "Run completed. Score: {}/100",
                    val["score"].as_u64().unwrap_or(0)
                );
            }
            _ => {}
        }
    }

    Ok(())
}

fn cmd_report_show(runs_dir: &Path, run_id: &str) -> Result<()> {
    let run_dir = runs_dir.join(run_id);
    if !run_dir.exists() {
        bail!("Run not found: {}", run_id);
    }
    let run = agentgauntlet_report::json::read_report(&run_dir)?;
    agentgauntlet_report::console::print_report(&run, &run_dir);
    Ok(())
}
