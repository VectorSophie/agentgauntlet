use thiserror::Error;

#[derive(Error, Debug)]
pub enum GauntletError {
    #[error("Scenario error: {0}")]
    Scenario(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Timeout after {ms}ms")]
    Timeout { ms: u64 },

    #[error("Evaluation error: {0}")]
    Evaluation(String),

    #[error("Report error: {0}")]
    Report(String),
}
