use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub category: String,
    pub severity: String,
    pub target: Target,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub tools: ToolConfig,
    pub steps: Vec<Step>,
}

fn default_timeout() -> u64 {
    30000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    #[serde(rename = "type")]
    pub kind: TargetKind,
    pub command: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TargetKind {
    Cli,
    Http,
    Builtin,
    /// Agent injected at runtime by `agentgauntlet scan`; target fields are ignored.
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolConfig {
    #[serde(default)]
    pub allowed: Vec<String>,
    #[serde(default)]
    pub forbidden: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub user: String,
    #[serde(default)]
    pub expect: Expectations,
    pub on_refusal: Option<EscalationStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationStrategy {
    Retry { rephrase: String },
    Escalate { next_turn: String },
    Pivot { alternative_attack: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Expectations {
    pub output: Option<OutputExpect>,
    pub tools: Option<ToolExpect>,
    pub memory: Option<MemoryExpect>,
    pub trajectory: Option<TrajectoryExpect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputExpect {
    #[serde(default)]
    pub must_contain: Vec<String>,
    #[serde(default)]
    pub must_not_contain: Vec<String>,
    #[serde(default)]
    pub regex_must_match: Vec<String>,
    #[serde(default)]
    pub regex_must_not_match: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolExpect {
    #[serde(default)]
    pub allowed_only: Vec<String>,
    #[serde(default)]
    pub forbidden: Vec<String>,
    #[serde(default)]
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryExpect {
    #[serde(default)]
    pub should_store: Vec<String>,
    #[serde(default)]
    pub should_not_store: Vec<String>,
    #[serde(default)]
    pub should_not_retrieve: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrajectoryExpect {
    #[serde(default)]
    pub must_not_change_goal: bool,
    #[serde(default)]
    pub must_not_escalate_permissions: bool,
}
