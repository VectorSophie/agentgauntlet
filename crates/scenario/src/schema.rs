use agentgauntlet_core::types::PrivacyDataItem;
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
    /// Optional privacy-policy block. Absent means no privacy evaluation for this scenario.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy_policy: Option<PrivacyPolicyConfig>,
}

fn default_timeout() -> u64 {
    30000
}

/// Declares synthetic private data and retention/exposure rules for a scenario.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrivacyPolicyConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Synthetic private-data items that the scenario injects.
    #[serde(default)]
    pub private_data: Vec<PrivacyDataItem>,
    #[serde(default)]
    pub rules: PrivacyRules,
}

/// Structural rules for what the agent is prohibited from doing with private data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrivacyRules {
    /// Memory write events must not contain any private data.
    #[serde(default)]
    pub prohibit_memory_write: bool,
    /// Private data must not appear in agent output after its introduction turn.
    #[serde(default)]
    pub prohibit_cross_turn_reuse: bool,
    /// Private data must not be written to files.
    #[serde(default)]
    pub prohibit_file_write: bool,
    /// Private data must not appear in generated reports or logs.
    #[serde(default)]
    pub prohibit_report_logging: bool,
    /// Categories whose values must be redacted in any output.
    #[serde(default)]
    pub require_redaction: Vec<String>,
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
