use crate::{Judge, JudgeContext, JudgeVerdict};
use anyhow::Result;
use async_trait::async_trait;

pub struct NullJudge;

#[async_trait]
impl Judge for NullJudge {
    async fn evaluate(&self, _ctx: &JudgeContext) -> Result<JudgeVerdict> {
        Ok(JudgeVerdict {
            complied: false,
            confidence: 0.0,
            severity: None,
            reasoning: "Null judge does not evaluate".to_string(),
        })
    }

    fn name(&self) -> &str {
        "none"
    }
}
