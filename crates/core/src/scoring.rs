use crate::types::{Finding, SecurityScore};

pub fn compute_score(findings: &[Finding]) -> SecurityScore {
    SecurityScore::compute(findings)
}

pub fn attack_succeeded(score: &SecurityScore) -> bool {
    score.high > 0 || score.critical > 0
}
