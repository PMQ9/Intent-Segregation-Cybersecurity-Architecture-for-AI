//! Red Team Attack Implementations
//!
//! Comprehensive attack mechanisms organized by phase:
//! - Phase 1: Direct Injection Attacks
//! - Phase 2: Indirect Injection Attacks
//! - Phase 3: Jailbreak Attacks
//! - Phase 4: Consensus-Breaking Attacks
//! - Phase 5: Adaptive Attacks

// Phase 1: Direct Injection Attacks
pub mod direct_injection;

// Phase 2: Indirect Injection Attacks
pub mod indirect_injection;

// Phase 3: Jailbreak Attacks
pub mod jailbreaks;

// Phase 4: Consensus-Breaking Attacks
pub mod consensus_breaking;

// Phase 5: Adaptive Attacks
// pub mod adaptive;

pub use direct_injection::{AttackPayload, AttackResult};
pub use indirect_injection::{
    IndirectAttackPayload, IndirectAttackResult,
    WebsiteInjectionAttack, EmailInjectionAttack,
    AgentInjectionAttack, MultimodalAttack
};
pub use jailbreaks::{
    JailbreakPayload, JailbreakResult,
    RoleplayAttack, MultiTurnAttack,
    WeakToStrongAttack, ObfuscationAttack
};
pub use consensus_breaking::{
    ConsensusAttackPayload, ConsensusAttackResult,
    ParserSpecificAttack, ParserVariant,
    VotingBypassAttack
};
