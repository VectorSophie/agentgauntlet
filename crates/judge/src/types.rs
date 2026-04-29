use agentgauntlet_core::types::{Finding, Severity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct JudgeContext {
    pub scenario_name: String,
    pub attack_intent: String,
    pub history: Vec<(String, String)>,
    pub current_user: String,
    pub current_agent: String,
    pub rule_findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeVerdict {
    pub complied: bool,
    pub confidence: f32,
    pub severity: Option<Severity>,
    pub reasoning: String,
    pub patch_suggestion: Option<String>,
}
