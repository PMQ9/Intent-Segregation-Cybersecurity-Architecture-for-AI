use thiserror::Error;

/// Result type for sacrificial LLM operations
pub type CogitatorResult<T> = Result<T, CogitatorError>;

/// Errors that can occur during sacrificial LLM testing
#[derive(Debug, Error)]
pub enum CogitatorError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Timeout error: cogitator took too long to respond")]
    TimeoutError,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Detection failed: {0}")]
    DetectionError(String),
}

/// Result of testing input on a sacrificial LLM
#[derive(Debug, Clone)]
pub struct CogitatorCorruptionTest {
    /// Name of the cogitator that ran the test
    pub cogitator_name: String,

    /// Was the input flagged as potentially corrupted/malicious?
    pub is_suspicious: bool,

    /// Risk score (0.0 = safe, 1.0 = definitely malicious)
    pub risk_score: f32,

    /// Specific attack indicators found
    pub attack_indicators: Vec<String>,

    /// Detailed analysis from the cogitator
    pub analysis: String,

    /// Time taken to process (ms)
    pub processing_time_ms: u128,
}

/// Consensus from multiple sacrificial LLMs
#[derive(Debug, Clone)]
pub struct CorruptionConsensus {
    /// Overall verdict: is input corrupted/malicious?
    pub is_corrupted: bool,

    /// Average risk score from all cogitators
    pub consensus_risk_score: f32,

    /// Number of cogitators that flagged as suspicious
    pub suspicious_count: usize,

    /// Total number of cogitators that ran
    pub total_cogitators: usize,

    /// Individual results from each cogitator
    pub individual_results: Vec<CogitatorCorruptionTest>,

    /// Combined analysis text
    pub combined_analysis: String,
}

/// Trait for sacrificial LLM cogitators
#[async_trait::async_trait]
pub trait SacrificialCogitator: Send + Sync {
    /// Test input for corruption/malicious content
    /// This is a lightweight, fast test to catch early indicators of attacks
    async fn test_for_corruption(
        &self,
        user_input: &str,
    ) -> CogitatorResult<CogitatorCorruptionTest>;

    /// Get the cogitator's name
    fn cogitator_name(&self) -> String;

    /// Check if this cogitator is properly configured
    fn is_configured(&self) -> bool;
}
