use agentgauntlet_core::scoring::attack_succeeded;
use agentgauntlet_demo::{get_demo_scenarios, DemoRunner};
use tempfile::TempDir;

#[tokio::test]
async fn demo_vulnerable_agent_fails_scenarios() {
    let tmp = TempDir::new().unwrap();
    let runner = DemoRunner::new(tmp.path().to_path_buf());
    let scenarios = get_demo_scenarios();

    assert!(!scenarios.is_empty(), "Demo scenarios should not be empty");

    let mut any_failed = false;
    for scenario in &scenarios {
        let run = runner.run_scenario(scenario).await.unwrap();

        // The vulnerable agent should produce findings
        if attack_succeeded(&run.score) {
            any_failed = true;
        }
    }

    // At least one scenario must fail — the vulnerable agent is designed to fail
    assert!(
        any_failed,
        "Vulnerable agent should trigger at least one high/critical finding"
    );
}

#[tokio::test]
async fn demo_score_below_threshold() {
    let tmp = TempDir::new().unwrap();
    let runner = DemoRunner::new(tmp.path().to_path_buf());
    let scenarios = get_demo_scenarios();

    let mut total_score: u32 = 0;
    let count = scenarios.len() as u32;

    for scenario in &scenarios {
        let run = runner.run_scenario(scenario).await.unwrap();
        total_score += run.score.score as u32;
    }

    let avg = total_score.checked_div(count).unwrap_or(100);
    assert!(
        avg < 70,
        "Average score for vulnerable agent should be below 70, got {}",
        avg
    );
}

#[tokio::test]
async fn scenario_loader_parses_demo_yaml() {
    let demo_dir = std::path::Path::new("../../scenarios/demo");
    if !demo_dir.exists() {
        // Run from workspace root context — adjust path
        return;
    }
    let files = agentgauntlet_scenario::find_scenarios(demo_dir).unwrap();
    for path in &files {
        let scenario = agentgauntlet_scenario::load_scenario(path).unwrap();
        agentgauntlet_scenario::validate(&scenario).unwrap();
    }
}
