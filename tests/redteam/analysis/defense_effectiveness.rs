//! Defense Effectiveness Analysis
//!
//! Analyzes the effectiveness of individual defense layers and components
//! in the Intent Segregation Architecture.

use std::collections::HashMap;

/// Defense layer in the Intent Segregation Architecture
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DefenseLayer {
    VaultOfTheForbiddenCant,
    MultiParserConsensus,
    PolicyComparator,
    HumanApproval,
    AuditLedger,
}

impl DefenseLayer {
    pub fn name(&self) -> &str {
        match self {
            DefenseLayer::VaultOfTheForbiddenCant => "Vault of the Forbidden Cant",
            DefenseLayer::MultiParserConsensus => "Multi-Parser Consensus",
            DefenseLayer::PolicyComparator => "Policy Comparator",
            DefenseLayer::HumanApproval => "Human-in-Loop Approval",
            DefenseLayer::AuditLedger => "Audit Ledger",
        }
    }
}

/// Defense layer effectiveness metrics
#[derive(Debug, Clone)]
pub struct DefenseEffectivenessAnalysis {
    pub layer_effectiveness: HashMap<DefenseLayer, f32>,
    pub layer_detection_rates: HashMap<DefenseLayer, f32>,
    pub layer_blocking_rates: HashMap<DefenseLayer, f32>,
    pub weakest_layer: Option<DefenseLayer>,
    pub strongest_layer: Option<DefenseLayer>,
}

#[derive(Debug, Clone)]
pub struct LayerTestResult {
    pub layer: DefenseLayer,
    pub attack_type: String,
    pub detected: bool,
    pub blocked: bool,
    pub confidence: f32,
}

impl DefenseEffectivenessAnalysis {
    /// Calculate effectiveness from test results
    pub fn calculate(results: &[LayerTestResult]) -> Self {
        let mut layer_stats: HashMap<DefenseLayer, (usize, usize, usize)> = HashMap::new();
        // (total, detected, blocked)

        for result in results {
            let (total, detected, blocked) = layer_stats.entry(result.layer.clone()).or_insert((0, 0, 0));
            *total += 1;
            if result.detected {
                *detected += 1;
            }
            if result.blocked {
                *blocked += 1;
            }
        }

        let mut layer_effectiveness = HashMap::new();
        let mut layer_detection_rates = HashMap::new();
        let mut layer_blocking_rates = HashMap::new();

        for (layer, (total, detected, blocked)) in layer_stats.iter() {
            let detection_rate = if *total > 0 {
                (*detected as f32) / (*total as f32)
            } else {
                0.0
            };

            let blocking_rate = if *total > 0 {
                (*blocked as f32) / (*total as f32)
            } else {
                0.0
            };

            let effectiveness = (detection_rate + blocking_rate) / 2.0;

            layer_effectiveness.insert(layer.clone(), effectiveness);
            layer_detection_rates.insert(layer.clone(), detection_rate);
            layer_blocking_rates.insert(layer.clone(), blocking_rate);
        }

        let weakest_layer = layer_effectiveness
            .iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, _)| k.clone());

        let strongest_layer = layer_effectiveness
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, _)| k.clone());

        DefenseEffectivenessAnalysis {
            layer_effectiveness,
            layer_detection_rates,
            layer_blocking_rates,
            weakest_layer,
            strongest_layer,
        }
    }

    /// Get layer efficiency score (0.0 to 1.0)
    pub fn get_layer_score(&self, layer: &DefenseLayer) -> f32 {
        self.layer_effectiveness.get(layer).copied().unwrap_or(0.0)
    }

    /// Check if all layers meet minimum effectiveness threshold
    pub fn meets_minimum_threshold(&self, threshold: f32) -> bool {
        self.layer_effectiveness.values().all(|&score| score >= threshold)
    }

    /// Get recommendation for weakest layer
    pub fn get_improvement_recommendation(&self) -> String {
        match &self.weakest_layer {
            Some(layer) => format!(
                "Recommended improvement: Strengthen {} (current effectiveness: {:.1}%)",
                layer.name(),
                self.get_layer_score(layer) * 100.0
            ),
            None => "All layers performing well".to_string(),
        }
    }

    /// Generate defense analysis summary
    pub fn generate_summary(&self) -> String {
        format!(
            "Defense Effectiveness Analysis:\nStrongest: {:?} ({:.1}%)\nWeakest: {:?} ({:.1}%)\nRecommendation: {}",
            self.strongest_layer.as_ref().map(|l| l.name()),
            self.strongest_layer.as_ref().map_or(0.0, |l| self.get_layer_score(l) * 100.0),
            self.weakest_layer.as_ref().map(|l| l.name()),
            self.weakest_layer.as_ref().map_or(0.0, |l| self.get_layer_score(l) * 100.0),
            self.get_improvement_recommendation()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effectiveness_calculation() {
        let results = vec![
            LayerTestResult {
                layer: DefenseLayer::VaultOfTheForbiddenCant,
                attack_type: "direct_injection".to_string(),
                detected: true,
                blocked: true,
                confidence: 0.9,
            },
            LayerTestResult {
                layer: DefenseLayer::VaultOfTheForbiddenCant,
                attack_type: "direct_injection".to_string(),
                detected: false,
                blocked: false,
                confidence: 0.1,
            },
        ];

        let analysis = DefenseEffectivenessAnalysis::calculate(&results);
        assert_eq!(
            analysis.get_layer_score(&DefenseLayer::VaultOfTheForbiddenCant),
            0.5
        );
    }

    #[test]
    fn test_minimum_threshold() {
        let mut analysis = DefenseEffectivenessAnalysis {
            layer_effectiveness: {
                let mut m = HashMap::new();
                m.insert(DefenseLayer::VaultOfTheForbiddenCant, 0.95);
                m.insert(DefenseLayer::MultiParserConsensus, 0.90);
                m
            },
            layer_detection_rates: HashMap::new(),
            layer_blocking_rates: HashMap::new(),
            weakest_layer: None,
            strongest_layer: None,
        };

        assert!(analysis.meets_minimum_threshold(0.85));
        assert!(!analysis.meets_minimum_threshold(0.95));
    }

    #[test]
    fn test_layer_names() {
        assert_eq!(
            DefenseLayer::VaultOfTheForbiddenCant.name(),
            "Vault of the Forbidden Cant"
        );
        assert_eq!(
            DefenseLayer::MultiParserConsensus.name(),
            "Multi-Parser Consensus"
        );
    }
}
