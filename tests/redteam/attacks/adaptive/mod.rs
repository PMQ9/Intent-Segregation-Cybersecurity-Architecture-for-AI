//! Phase 5: Adaptive Attacks
//!
//! This module contains adaptive attack mechanisms that optimize against the system
//! through various optimization strategies. These represent state-of-the-art LLM security
//! research as of November 2025.
//!
//! # Attack Categories
//!
//! 1. **RL-Based Attacks**: Reinforcement learning optimization across 32 sessions × 5 rounds
//!    - Simulates agents learning system weaknesses through interaction
//!    - Each session attempts 5 refinement rounds
//!    - Total: 160 attack variants
//!
//! 2. **Search-Based Attacks**: LLM-generated variant optimization (10 variants × 100 iterations)
//!    - Uses LLM to generate attack variants
//!    - Judge LLM scores effectiveness
//!    - Iterative refinement across 100 iterations
//!    - Total: 1000 variants generated
//!
//! 3. **Data-Flow Attacks**: Data fields become control flow instructions
//!    - Input validation bypasses
//!    - Parameter injection
//!    - Command injection in data fields
//!    - 12+ test cases
//!
//! 4. **Cascade Attacks**: Multi-step escalation chains
//!    - Initial compromise enables further access
//!    - Service-to-service escalation
//!    - Privilege escalation patterns
//!    - 10+ test cases
//!
//! # Defensive Testing Context
//!
//! These attacks are generated to test system defenses. The goal is to verify that
//! adaptive optimization strategies cannot break the Intent Segregation Architecture's
//! multiple defensive layers:
//! - Zero-trust input validation (Vault of the Forbidden Cant)
//! - Multi-parser consensus (requires ≥95% agreement)
//! - Policy enforcement
//! - Human-in-the-loop escalation

pub mod rl_based;
pub mod search_based;
pub mod data_flow;
pub mod cascade;

pub use rl_based::RLBasedAttack;
pub use search_based::SearchBasedAttack;
pub use data_flow::DataFlowAttack;
pub use cascade::CascadeAttack;

use std::fmt;

/// Represents an adaptive attack payload with optimization metadata
#[derive(Debug, Clone)]
pub struct AdaptiveAttackPayload {
    pub attack_type: String,
    pub category: String,
    /// The attack payload
    pub payload: String,
    /// Base payload before optimization
    pub base_payload: String,
    /// Optimization round number (0 = base, 1-N = optimized)
    pub optimization_round: usize,
    /// Which session this is from (for RL-based)
    pub session_id: Option<usize>,
    /// Mutation/variant index
    pub variant_index: usize,
    /// Estimated effectiveness (0.0 to 1.0)
    pub estimated_effectiveness: f32,
    /// Description of optimization strategy
    pub optimization_strategy: String,
    /// Should this payload be blocked?
    pub should_block: bool,
}

impl AdaptiveAttackPayload {
    pub fn new(
        attack_type: impl Into<String>,
        category: impl Into<String>,
        payload: impl Into<String>,
        base_payload: impl Into<String>,
        optimization_round: usize,
        strategy: impl Into<String>,
    ) -> Self {
        Self {
            attack_type: attack_type.into(),
            category: category.into(),
            payload: payload.into(),
            base_payload: base_payload.into(),
            optimization_round,
            session_id: None,
            variant_index: 0,
            estimated_effectiveness: 0.5,
            optimization_strategy: strategy.into(),
            should_block: true,
        }
    }

    pub fn with_session(mut self, session_id: usize) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_variant(mut self, variant_index: usize) -> Self {
        self.variant_index = variant_index;
        self
    }

    pub fn with_effectiveness(mut self, effectiveness: f32) -> Self {
        self.estimated_effectiveness = effectiveness.max(0.0).min(1.0);
        self
    }
}

impl fmt::Display for AdaptiveAttackPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let session_info = self.session_id.map_or_else(
            || "N/A".to_string(),
            |s| format!("Session {}", s),
        );
        write!(
            f,
            "AdaptiveAttack[{}::{}]\nRound: {}, Variant: {}, {}\nStrategy: {}\nEffectiveness: {:.1}%\nPayload: {}",
            self.attack_type, self.category, self.optimization_round, self.variant_index, session_info,
            self.optimization_strategy, self.estimated_effectiveness * 100.0, self.payload
        )
    }
}

/// Result of testing an adaptive attack
#[derive(Debug, Clone)]
pub struct AdaptiveAttackResult {
    pub payload: AdaptiveAttackPayload,
    pub detected: bool,
    pub blocked: bool,
    pub confidence: f32,
    pub bypass_method: Option<String>,
}

impl AdaptiveAttackResult {
    pub fn new(payload: AdaptiveAttackPayload, detected: bool, blocked: bool, confidence: f32) -> Self {
        Self {
            payload,
            detected,
            blocked,
            confidence,
            bypass_method: None,
        }
    }

    pub fn with_bypass_method(mut self, method: String) -> Self {
        self.bypass_method = Some(method);
        self
    }

    /// Did the adaptive attack bypass defenses?
    pub fn bypassed(&self) -> bool {
        self.payload.should_block && !self.detected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_payload_creation() {
        let payload = AdaptiveAttackPayload::new(
            "rl_based",
            "optimization",
            "variant_payload".to_string(),
            "base_payload".to_string(),
            5,
            "RL refinement",
        );
        assert_eq!(payload.optimization_round, 5);
        assert!(payload.should_block);
    }

    #[test]
    fn test_with_session_and_variant() {
        let payload = AdaptiveAttackPayload::new(
            "rl_based",
            "optimization",
            "p".to_string(),
            "b".to_string(),
            1,
            "test",
        )
        .with_session(10)
        .with_variant(5)
        .with_effectiveness(0.75);

        assert_eq!(payload.session_id, Some(10));
        assert_eq!(payload.variant_index, 5);
        assert_eq!(payload.estimated_effectiveness, 0.75);
    }

    #[test]
    fn test_adaptive_attack_result() {
        let payload = AdaptiveAttackPayload::new(
            "search_based",
            "variant",
            "p".to_string(),
            "b".to_string(),
            3,
            "judge optimization",
        );
        let result = AdaptiveAttackResult::new(payload, false, false, 0.0);
        assert!(result.bypassed());
    }
}
