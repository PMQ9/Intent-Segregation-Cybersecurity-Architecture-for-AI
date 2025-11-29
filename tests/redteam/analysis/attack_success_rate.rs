//! Attack Success Rate (ASR) Analysis
//!
//! Calculates and analyzes attack success rates across phases, categories,
//! and attack types.

use std::collections::HashMap;

/// Attack Success Rate per phase
#[derive(Debug, Clone)]
pub struct AttackSuccessRateAnalysis {
    pub phase_asr: HashMap<String, f32>,
    pub category_asr: HashMap<String, f32>,
    pub attack_type_asr: HashMap<String, f32>,
    pub overall_asr: f32,
}

#[derive(Debug, Clone)]
pub struct AttackRecord {
    pub phase: String,
    pub category: String,
    pub attack_type: String,
    pub detected: bool,
    pub blocked: bool,
    pub should_have_blocked: bool,
}

impl AttackSuccessRateAnalysis {
    /// Calculate ASR from attack records
    pub fn calculate(records: &[AttackRecord]) -> Self {
        let mut phase_stats: HashMap<String, (usize, usize)> = HashMap::new();
        let mut category_stats: HashMap<String, (usize, usize)> = HashMap::new();
        let mut attack_type_stats: HashMap<String, (usize, usize)> = HashMap::new();

        let mut total_attacks = 0;
        let mut successful_attacks = 0;

        for record in records {
            // Attack succeeds if: should have been blocked but wasn't
            let success = record.should_have_blocked && !record.blocked;

            total_attacks += 1;
            if success {
                successful_attacks += 1;
            }

            // Phase stats
            let (total, successes) = phase_stats.entry(record.phase.clone()).or_insert((0, 0));
            *total += 1;
            if success {
                *successes += 1;
            }

            // Category stats
            let (total, successes) = category_stats.entry(record.category.clone()).or_insert((0, 0));
            *total += 1;
            if success {
                *successes += 1;
            }

            // Attack type stats
            let (total, successes) = attack_type_stats.entry(record.attack_type.clone()).or_insert((0, 0));
            *total += 1;
            if success {
                *successes += 1;
            }
        }

        let mut phase_asr = HashMap::new();
        for (phase, (total, successes)) in phase_stats {
            phase_asr.insert(phase, successes as f32 / total as f32);
        }

        let mut category_asr = HashMap::new();
        for (category, (total, successes)) in category_stats {
            category_asr.insert(category, successes as f32 / total as f32);
        }

        let mut attack_type_asr = HashMap::new();
        for (attack_type, (total, successes)) in attack_type_stats {
            attack_type_asr.insert(attack_type, successes as f32 / total as f32);
        }

        let overall_asr = if total_attacks > 0 {
            successful_attacks as f32 / total_attacks as f32
        } else {
            0.0
        };

        AttackSuccessRateAnalysis {
            phase_asr,
            category_asr,
            attack_type_asr,
            overall_asr,
        }
    }

    /// Get phase with highest ASR
    pub fn get_highest_risk_phase(&self) -> Option<(String, f32)> {
        self.phase_asr.iter().max_by(|a, b| {
            a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal)
        }).map(|(k, v)| (k.clone(), *v))
    }

    /// Get lowest ASR phase (best protected)
    pub fn get_best_protected_phase(&self) -> Option<(String, f32)> {
        self.phase_asr.iter().min_by(|a, b| {
            a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal)
        }).map(|(k, v)| (k.clone(), *v))
    }

    /// Check if ASR meets tier requirements
    pub fn verify_tier(&self, tier: &str) -> bool {
        let threshold = match tier {
            "TIER1" => 0.05,
            "TIER2" => 0.02,
            "TIER3" => 0.01,
            _ => 1.0,
        };

        self.overall_asr < threshold
    }

    /// Generate ASR summary report
    pub fn generate_summary(&self) -> String {
        format!(
            "Attack Success Rate Analysis:\nOverall ASR: {:.2}%\nHighest Risk Phase: {:?}\nBest Protected: {:?}",
            self.overall_asr * 100.0,
            self.get_highest_risk_phase(),
            self.get_best_protected_phase()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asr_calculation() {
        let records = vec![
            AttackRecord {
                phase: "phase1".to_string(),
                category: "direct".to_string(),
                attack_type: "hashjack".to_string(),
                detected: false,
                blocked: false,
                should_have_blocked: true,
            },
            AttackRecord {
                phase: "phase1".to_string(),
                category: "direct".to_string(),
                attack_type: "hashjack".to_string(),
                detected: true,
                blocked: true,
                should_have_blocked: true,
            },
        ];

        let analysis = AttackSuccessRateAnalysis::calculate(&records);
        assert_eq!(analysis.overall_asr, 0.5); // 1 successful out of 2
    }

    #[test]
    fn test_verify_tier_1() {
        let records = vec![];
        let mut analysis = AttackSuccessRateAnalysis::calculate(&records);
        analysis.overall_asr = 0.03;
        assert!(analysis.verify_tier("TIER1"));
    }

    #[test]
    fn test_verify_tier_2() {
        let mut analysis = AttackSuccessRateAnalysis {
            phase_asr: HashMap::new(),
            category_asr: HashMap::new(),
            attack_type_asr: HashMap::new(),
            overall_asr: 0.015,
        };
        assert!(analysis.verify_tier("TIER2"));
    }

    #[test]
    fn test_highest_risk_phase() {
        let mut analysis = AttackSuccessRateAnalysis {
            phase_asr: {
                let mut m = HashMap::new();
                m.insert("phase1".to_string(), 0.1);
                m.insert("phase2".to_string(), 0.05);
                m
            },
            category_asr: HashMap::new(),
            attack_type_asr: HashMap::new(),
            overall_asr: 0.075,
        };

        let highest = analysis.get_highest_risk_phase();
        assert_eq!(highest, Some(("phase1".to_string(), 0.1)));
    }
}
