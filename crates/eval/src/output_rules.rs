use agentgauntlet_core::types::{Finding, Severity};
use agentgauntlet_scenario::schema::OutputExpect;
use regex::Regex;

pub fn evaluate(output: &str, expect: &OutputExpect, turn_index: usize) -> Vec<Finding> {
    let mut findings = Vec::new();

    for phrase in &expect.must_contain {
        if !output.to_lowercase().contains(&phrase.to_lowercase()) {
            findings.push(Finding {
                rule_id: "output.must_contain".to_string(),
                severity: Severity::Medium,
                message: format!("Output missing required text: {phrase}"),
                turn_index,
                evidence: output.chars().take(200).collect(),
            });
        }
    }

    for phrase in &expect.must_not_contain {
        if output.to_lowercase().contains(&phrase.to_lowercase()) {
            findings.push(Finding {
                rule_id: "output.must_not_contain".to_string(),
                severity: Severity::High,
                message: format!("Output contained forbidden text: {phrase}"),
                turn_index,
                evidence: extract_evidence(output, phrase),
            });
        }
    }

    for pattern in &expect.regex_must_match {
        match Regex::new(pattern) {
            Ok(re) => {
                if !re.is_match(output) {
                    findings.push(Finding {
                        rule_id: "output.regex_must_match".to_string(),
                        severity: Severity::Medium,
                        message: format!("Output did not match required pattern: {pattern}"),
                        turn_index,
                        evidence: output.chars().take(200).collect(),
                    });
                }
            }
            Err(e) => {
                tracing::warn!("Invalid regex '{}': {}", pattern, e);
            }
        }
    }

    for pattern in &expect.regex_must_not_match {
        match Regex::new(pattern) {
            Ok(re) => {
                if re.is_match(output) {
                    let mat = re
                        .find(output)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();
                    findings.push(Finding {
                        rule_id: "output.regex_must_not_match".to_string(),
                        severity: Severity::High,
                        message: format!("Output matched forbidden pattern: {pattern}"),
                        turn_index,
                        evidence: mat,
                    });
                }
            }
            Err(e) => {
                tracing::warn!("Invalid regex '{}': {}", pattern, e);
            }
        }
    }

    findings
}

fn extract_evidence(output: &str, phrase: &str) -> String {
    let lower_output = output.to_lowercase();
    let lower_phrase = phrase.to_lowercase();
    if let Some(pos) = lower_output.find(&lower_phrase) {
        let start = pos.saturating_sub(30);
        let end = (pos + phrase.len() + 30).min(output.len());
        format!("...{}...", &output[start..end])
    } else {
        output.chars().take(200).collect()
    }
}
