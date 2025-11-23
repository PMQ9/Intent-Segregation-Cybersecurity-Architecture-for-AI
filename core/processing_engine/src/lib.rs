use intent_schema::{
    Action, DocumentSummary, Expert, Intent, Proposal, ProposalSection, ProcessingMetadata,
    ProcessingResult, Expertise,
};
use chrono::Utc;
use serde_json::json;
use std::time::Instant;
use thiserror::Error;
use tracing::{info, warn};

/// Errors that can occur during processing
#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Unsupported action: {0:?}")]
    UnsupportedAction(Action),

    #[error("Invalid intent: {0}")]
    InvalidIntent(String),

    #[error("Processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// The main processing engine that executes trusted intents
///
/// This engine ensures that:
/// - All operations are type-safe and use typed function calls
/// - No free-form LLM calls can execute privileged actions
/// - All intents are validated before execution
/// - Results are structured and auditable
pub struct ProcessingEngine {
    /// Configuration for the engine
    config: EngineConfig,
}

/// Configuration for the processing engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Enable verbose logging
    pub verbose: bool,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            max_execution_time_ms: 30_000, // 30 seconds
        }
    }
}

impl ProcessingEngine {
    /// Create a new processing engine with default configuration
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
        }
    }

    /// Create a new processing engine with custom configuration
    pub fn with_config(config: EngineConfig) -> Self {
        Self { config }
    }

    /// Execute a trusted intent and return a structured result
    ///
    /// This is the main entry point for processing. It dispatches to
    /// type-safe functions based on the intent action.
    ///
    /// # Security Guarantees
    /// - Only predefined actions can be executed
    /// - All parameters are validated
    /// - No raw prompts or unstructured commands are accepted
    /// - All operations are logged and traceable
    pub async fn execute(&self, intent: &Intent) -> Result<ProcessingResult, ProcessingError> {
        let start_time = Instant::now();
        let started_at = Utc::now();

        info!(
            "Executing intent: action={:?}, topic={:?}",
            intent.action, intent.topic
        );

        // Dispatch to the appropriate typed function based on the action
        let result = match &intent.action {
            Action::FindExperts => self.execute_find_experts(intent).await,
            Action::Summarize => self.execute_summarize(intent).await,
            Action::DraftProposal => self.execute_draft_proposal(intent).await,
            Action::AnalyzeDocument => self.execute_analyze_document(intent).await,
            Action::GenerateReport => self.execute_generate_report(intent).await,
            Action::SearchKnowledge => self.execute_search_knowledge(intent).await,
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let completed_at = Utc::now();

        match result {
            Ok((function_name, data, warnings)) => {
                info!("Intent executed successfully in {}ms", duration_ms);

                Ok(ProcessingResult::success(
                    intent.action.clone(),
                    data,
                    ProcessingMetadata {
                        started_at,
                        completed_at,
                        duration_ms,
                        function_called: function_name,
                        warnings,
                    },
                ))
            }
            Err(e) => {
                warn!("Intent execution failed: {}", e);

                Ok(ProcessingResult::failure(
                    intent.action.clone(),
                    e.to_string(),
                    ProcessingMetadata {
                        started_at,
                        completed_at,
                        duration_ms,
                        function_called: "unknown".to_string(),
                        warnings: vec![],
                    },
                ))
            }
        }
    }

    /// Find experts matching the intent criteria (MOCK)
    ///
    /// This is a typed function call - no free-form LLM prompting
    async fn execute_find_experts(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        let experts = find_experts(
            intent.topic.clone(),
            intent.expertise.clone(),
            intent.constraints.max_results.unwrap_or(10),
            intent.constraints.max_budget,
        );

        let data = json!({ "experts": experts, "count": experts.len() });

        Ok(("find_experts".to_string(), data, vec![]))
    }

    /// Summarize a document (MOCK)
    async fn execute_summarize(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        let document_refs = intent
            .content_refs
            .as_ref()
            .ok_or_else(|| ProcessingError::InvalidIntent("No content refs provided".to_string()))?;

        if document_refs.is_empty() {
            return Err(ProcessingError::InvalidIntent(
                "No documents to summarize".to_string(),
            ));
        }

        let summary = summarize_document(&document_refs[0], intent.topic.clone());

        let data = json!({ "summary": summary });

        Ok(("summarize_document".to_string(), data, vec![]))
    }

    /// Draft a proposal (MOCK)
    async fn execute_draft_proposal(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        let proposal = draft_proposal(
            intent.topic.clone(),
            intent.expertise.clone(),
            intent.constraints.max_budget,
        );

        let data = json!({ "proposal": proposal });

        let mut warnings = vec![];
        if proposal.estimated_budget.is_none() {
            warnings.push("Budget estimation not available".to_string());
        }

        Ok(("draft_proposal".to_string(), data, warnings))
    }

    /// Analyze a document (MOCK)
    async fn execute_analyze_document(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        let analysis = json!({
            "status": "analyzed",
            "topic": intent.topic,
            "complexity": "medium",
            "key_findings": ["Finding 1", "Finding 2", "Finding 3"]
        });

        Ok(("analyze_document".to_string(), analysis, vec![]))
    }

    /// Generate a report (MOCK)
    async fn execute_generate_report(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        let report = json!({
            "title": format!("Report: {}", intent.topic.as_deref().unwrap_or("Untitled")),
            "sections": [
                {"heading": "Executive Summary", "content": "..."},
                {"heading": "Detailed Analysis", "content": "..."},
                {"heading": "Recommendations", "content": "..."}
            ],
            "generated_at": Utc::now()
        });

        Ok(("generate_report".to_string(), report, vec![]))
    }

    /// Search knowledge base (MOCK)
    async fn execute_search_knowledge(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        let results = json!({
            "query": intent.topic,
            "results": [
                {"id": "doc1", "title": "Sample Document 1", "relevance": 0.95},
                {"id": "doc2", "title": "Sample Document 2", "relevance": 0.87},
            ],
            "total_count": 2
        });

        Ok(("search_knowledge".to_string(), results, vec![]))
    }
}

impl Default for ProcessingEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// MOCK IMPLEMENTATION FUNCTIONS
//
// These are placeholder implementations that demonstrate the type-safe
// function call pattern. In production, these would integrate with:
// - Real expert databases
// - Document processing pipelines
// - LLM APIs with structured outputs
// - Knowledge bases
//
// IMPORTANT: Notice that none of these functions accept raw prompts.
// All inputs are typed and validated.
// ============================================================================

/// Mock function to find experts
///
/// In production, this would query a database or API
fn find_experts(
    topic: Option<String>,
    expertise: Vec<Expertise>,
    max_results: u32,
    max_budget: Option<u64>,
) -> Vec<Expert> {
    let mut experts = vec![
        Expert {
            id: "exp_001".to_string(),
            name: "Dr. Sarah Chen".to_string(),
            expertise: vec!["security".to_string(), "cloud".to_string()],
            availability: true,
            hourly_rate: 250,
            confidence_score: 0.95,
            bio: Some("Expert in cloud security with 15 years of experience".to_string()),
            years_experience: Some(15),
        },
        Expert {
            id: "exp_002".to_string(),
            name: "James Rodriguez".to_string(),
            expertise: vec!["machine_learning".to_string(), "data_science".to_string()],
            availability: true,
            hourly_rate: 200,
            confidence_score: 0.88,
            bio: Some("ML researcher and practitioner".to_string()),
            years_experience: Some(10),
        },
        Expert {
            id: "exp_003".to_string(),
            name: "Emily Watson".to_string(),
            expertise: vec!["embedded".to_string(), "security".to_string()],
            availability: false,
            hourly_rate: 275,
            confidence_score: 0.92,
            bio: Some("Embedded systems security specialist".to_string()),
            years_experience: Some(12),
        },
    ];

    // Filter by budget if specified
    if let Some(budget) = max_budget {
        experts.retain(|e| e.hourly_rate <= budget);
    }

    // Limit results
    experts.truncate(max_results as usize);

    experts
}

/// Mock function to summarize a document
///
/// In production, this would call a document processing service
fn summarize_document(document_id: &str, topic: Option<String>) -> DocumentSummary {
    DocumentSummary {
        document_id: document_id.to_string(),
        title: format!("Document: {}", document_id),
        summary: format!(
            "This document covers {}. It provides comprehensive analysis \
             and actionable recommendations for stakeholders.",
            topic.unwrap_or_else(|| "various topics".to_string())
        ),
        key_points: vec![
            "Key finding 1: Market analysis shows strong growth potential".to_string(),
            "Key finding 2: Risk mitigation strategies are essential".to_string(),
            "Key finding 3: Timeline estimates are optimistic but achievable".to_string(),
        ],
        word_count: 2500,
        confidence: 0.89,
        generated_at: Utc::now(),
    }
}

/// Mock function to draft a proposal
///
/// In production, this would use a structured generation pipeline
fn draft_proposal(
    topic: Option<String>,
    expertise: Vec<Expertise>,
    budget: Option<u64>,
) -> Proposal {
    let topic_str = topic.unwrap_or_else(|| "Project Proposal".to_string());

    Proposal {
        id: format!("prop_{}", uuid::Uuid::new_v4()),
        title: format!("Proposal: {}", topic_str),
        sections: vec![
            ProposalSection {
                heading: "Executive Summary".to_string(),
                content: format!(
                    "This proposal outlines a comprehensive approach to {}. \
                     We bring together experts in {:?} to deliver exceptional results.",
                    topic_str, expertise
                ),
                order: 1,
            },
            ProposalSection {
                heading: "Scope of Work".to_string(),
                content: "Detailed breakdown of deliverables, milestones, and timeline.".to_string(),
                order: 2,
            },
            ProposalSection {
                heading: "Team and Expertise".to_string(),
                content: "Our team comprises industry-leading experts with proven track records."
                    .to_string(),
                order: 3,
            },
            ProposalSection {
                heading: "Budget and Timeline".to_string(),
                content: format!(
                    "Estimated budget: {}. Timeline: 12-16 weeks.",
                    budget
                        .map(|b| format!("${}", b))
                        .unwrap_or_else(|| "TBD".to_string())
                ),
                order: 4,
            },
        ],
        created_at: Utc::now(),
        estimated_budget: budget,
        timeline_weeks: Some(14),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intent_schema::{Action, Constraints, Expertise, Intent};

    #[tokio::test]
    async fn test_execute_find_experts() {
        let engine = ProcessingEngine::new();

        let intent = Intent {
            action: Action::FindExperts,
            topic: Some("supply_chain_risk".to_string()),
            expertise: vec![Expertise::Security],
            constraints: intent_schema::Constraints {
                max_budget: Some(300),
                max_results: Some(5),
                ..Default::default()
            },
            content_refs: Some(vec![]),
            metadata: Some(intent_schema::IntentMetadata {
                user_id: "test_user".to_string(),
                session_id: "test_session".to_string(),
            }),
        };

        let result = engine.execute(&intent).await.unwrap();

        assert!(result.success);
        assert_eq!(result.action, Action::FindExperts);
        assert!(result.data.get("experts").is_some());
        assert!(result.metadata.duration_ms < 1000);
    }

    #[tokio::test]
    async fn test_execute_summarize() {
        let engine = ProcessingEngine::new();

        let intent = Intent {
            action: Action::Summarize,
            topic: Some("cybersecurity trends".to_string()),
            expertise: vec![],
            constraints: Default::default(),
            content_refs: Some(vec!["doc_123".to_string()]),
            metadata: Some(intent_schema::IntentMetadata {
                user_id: "test_user".to_string(),
                session_id: "test_session".to_string(),
            }),
        };

        let result = engine.execute(&intent).await.unwrap();

        assert!(result.success);
        assert_eq!(result.action, Action::Summarize);
        assert!(result.data.get("summary").is_some());
    }

    #[tokio::test]
    async fn test_execute_draft_proposal() {
        let engine = ProcessingEngine::new();

        let intent = Intent {
            action: Action::DraftProposal,
            topic: Some("AI integration project".to_string()),
            expertise: vec![Expertise::MachineLearning, Expertise::Security],
            constraints: intent_schema::Constraints {
                max_budget: Some(50000),
                ..Default::default()
            },
            content_refs: Some(vec![]),
            metadata: Some(intent_schema::IntentMetadata {
                user_id: "test_user".to_string(),
                session_id: "test_session".to_string(),
            }),
        };

        let result = engine.execute(&intent).await.unwrap();

        assert!(result.success);
        assert_eq!(result.action, Action::DraftProposal);
        assert!(result.data.get("proposal").is_some());
    }

    #[tokio::test]
    async fn test_no_raw_prompts() {
        // This test demonstrates that the engine CANNOT accept raw prompts
        // All inputs must be structured Intents

        let engine = ProcessingEngine::new();

        // We can only pass typed Intent structures
        let valid_intent = Intent {
            action: Action::FindExperts,
            topic: Some("blockchain security".to_string()),
            expertise: vec![Expertise::Security],
            constraints: Default::default(),
            content_refs: Some(vec![]),
            metadata: Some(intent_schema::IntentMetadata {
                user_id: "test_user".to_string(),
                session_id: "test_session".to_string(),
            }),
        };

        let result = engine.execute(&valid_intent).await.unwrap();
        assert!(result.success);

        // The following would NOT compile (uncomment to verify):
        // let raw_prompt = "Find me some experts";
        // engine.execute(raw_prompt).await; // ❌ Type error!
    }

    #[test]
    fn test_find_experts_filters_by_budget() {
        let experts = find_experts(
            Some("security".to_string()),
            vec![Expertise::Security],
            10,
            Some(250), // Only experts charging ≤ $250/hr
        );

        assert!(experts.len() <= 2);
        assert!(experts.iter().all(|e| e.hourly_rate <= 250));
    }

    #[test]
    fn test_summarize_document_structure() {
        let summary = summarize_document("doc_456", Some("AI ethics".to_string()));

        assert_eq!(summary.document_id, "doc_456");
        assert!(summary.summary.contains("AI ethics"));
        assert!(!summary.key_points.is_empty());
        assert!(summary.confidence > 0.0 && summary.confidence <= 1.0);
    }

    #[test]
    fn test_draft_proposal_structure() {
        let proposal = draft_proposal(
            Some("Cloud migration".to_string()),
            vec![Expertise::Cloud, Expertise::DevOps],
            Some(100000),
        );

        assert!(proposal.title.contains("Cloud migration"));
        assert_eq!(proposal.sections.len(), 4);
        assert_eq!(proposal.estimated_budget, Some(100000));
        assert!(proposal.timeline_weeks.is_some());
    }
}
