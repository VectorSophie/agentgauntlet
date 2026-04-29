use agentgauntlet_core::types::{Run, Severity};
use std::path::Path;

pub fn print_report(run: &Run, run_dir: &Path) {
    println!();
    println!("AgentGauntlet Report: {}", run.scenario_name);
    println!("{}", "=".repeat(50));
    println!();
    println!(
        "Score: {}/100  {}",
        run.score.score,
        run.score.label.to_uppercase()
    );
    println!();

    if run.findings.is_empty() {
        println!("No findings — scenario passed.");
    } else {
        println!("Findings:");
        for finding in &run.findings {
            let sev = match finding.severity {
                Severity::Critical => "CRITICAL",
                Severity::High => "HIGH",
                Severity::Medium => "MEDIUM",
                Severity::Low => "LOW",
            };
            println!(
                "  [{sev}] turn_{} {}",
                finding.turn_index + 1,
                finding.rule_id
            );
            println!("    {}", finding.message);
            if !finding.evidence.is_empty() {
                println!("    Evidence: {}", finding.evidence);
            }
        }
    }

    println!();
    println!("Artifacts:");
    println!("  {}", run_dir.join("report.json").display());
    println!("  {}", run_dir.join("transcript.md").display());
    println!("  {}", run_dir.join("trace.jsonl").display());
    println!();
}

pub fn print_summary_line(scenario_name: &str, run: &Run) {
    let status = if run.score.critical > 0 || run.score.high > 0 {
        "[FAIL]"
    } else if run.score.medium > 0 {
        "[WARN]"
    } else {
        "[PASS]"
    };
    println!(
        "  {} {} — score={}/100 critical={} high={} medium={} low={}",
        status,
        scenario_name,
        run.score.score,
        run.score.critical,
        run.score.high,
        run.score.medium,
        run.score.low,
    );
}
