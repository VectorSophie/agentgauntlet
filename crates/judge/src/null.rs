use crate::{Judge, JudgeContext, JudgeVerdict};
use anyhow::Result;
use async_trait::async_trait;

pub struct NullJudge;

#[async_trait]
impl Judge for NullJudge {
    async fn evaluate(&self, _ctx: &JudgeContext) -> Result<JudgeVerdict> {
        Ok(JudgeVerdict {
            complied: false,
            confidence: 1.0,
            severity: None,
            reasoning: "Rule-based evaluator mode: relying on explicit findings.".to_string(),
            patch_suggestion: None,
        })
    }

    fn name(&self) -> &str {
        "none"
    }
}
