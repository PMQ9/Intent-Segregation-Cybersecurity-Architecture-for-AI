//! Test Orchestration and Benchmark Runners
//!
//! Coordinates test execution across phases, collects metrics, and generates reports.

use std::collections::HashMap;
use super::dashboard::MetricsDashboard;
use super::metrics::{AggregatedMetrics, MetricsSnapshot};

/// Phase runner configuration
#[derive(Debug, Clone)]
pub struct PhaseConfig {
    /// Phase identifier (e.g., "phase_1_direct_injection")
    pub phase_id: String,
    /// Phase name for display
    pub phase_name: String,
    /// Number of test cases in this phase
    pub test_count: usize,
    /// Whether to run this phase
    pub enabled: bool,
}

impl PhaseConfig {
    pub fn new(phase_id: String, phase_name: String, test_count: usize) -> Self {
        Self {
            phase_id,
            phase_name,
            test_count,
            enabled: true,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Red team benchmark runner
pub struct BenchmarkRunner {
    /// Dashboard instance
    pub dashboard: MetricsDashboard,
    /// Phase configurations
    pub phases: HashMap<String, PhaseConfig>,
    /// Collected metrics per phase
    pub phase_results: HashMap<String, Vec<MetricsSnapshot>>,
    /// Overall test count
    pub total_tests: usize,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new() -> Self {
        Self {
            dashboard: MetricsDashboard::new(),
            phases: HashMap::new(),
            phase_results: HashMap::new(),
            total_tests: 0,
        }
    }

    /// Create with custom run ID
    pub fn with_run_id(run_id: String) -> Self {
        Self {
            dashboard: MetricsDashboard::with_id(run_id),
            phases: HashMap::new(),
            phase_results: HashMap::new(),
            total_tests: 0,
        }
    }

    /// Register a phase for execution
    pub fn register_phase(&mut self, config: PhaseConfig) {
        if config.enabled {
            self.total_tests += config.test_count;
        }
        self.phases.insert(config.phase_id.clone(), config);
    }

    /// Register multiple phases
    pub fn register_phases(&mut self, configs: Vec<PhaseConfig>) {
        for config in configs {
            self.register_phase(config);
        }
    }

    /// Add metrics snapshot for a phase
    pub fn add_snapshot(&mut self, phase_id: &str, snapshot: MetricsSnapshot) {
        self.dashboard.add_snapshot(snapshot.clone());

        self.phase_results
            .entry(phase_id.to_string())
            .or_insert_with(Vec::new)
            .push(snapshot);
    }

    /// Finalize phase metrics
    pub fn finalize_phase(&mut self, phase_id: &str) {
        if let Some(snapshots) = self.phase_results.get(phase_id) {
            if !snapshots.is_empty() {
                let metrics = self.aggregate_snapshots(snapshots);
                self.dashboard.add_phase(phase_id.to_string(), metrics);
            }
        }
    }

    /// Aggregate snapshots into metrics
    fn aggregate_snapshots(&self, snapshots: &[MetricsSnapshot]) -> AggregatedMetrics {
        if snapshots.is_empty() {
            return AggregatedMetrics::default();
        }

        let attack_successes = snapshots.iter().filter(|s| s.attack_succeeded).count();
        let benign_rejections = snapshots.iter().filter(|s| s.benign_rejected).count();
        let vault_detections = snapshots.iter().filter(|s| s.vault_detected).count();
        let voting_conflicts = snapshots.iter().filter(|s| s.voting_conflict).count();
        let policy_approvals = snapshots.iter().filter(|s| s.policy_approved).count();
        let benign_corrects = snapshots.iter().filter(|s| s.benign_correct).count();

        let avg_latency = if snapshots.is_empty() {
            std::time::Duration::ZERO
        } else {
            let total: std::time::Duration = snapshots.iter().map(|s| s.latency).sum();
            total / snapshots.len() as u32
        };

        let mut latencies: Vec<_> = snapshots.iter().map(|s| s.latency).collect();
        latencies.sort();

        let p95_latency = if latencies.is_empty() {
            std::time::Duration::ZERO
        } else {
            let idx = (latencies.len() as f64 * 0.95) as usize;
            latencies[idx.min(latencies.len() - 1)]
        };

        let p99_latency = if latencies.is_empty() {
            std::time::Duration::ZERO
        } else {
            let idx = (latencies.len() as f64 * 0.99) as usize;
            latencies[idx.min(latencies.len() - 1)]
        };

        let total_tokens: usize = snapshots.iter().map(|s| s.tokens_used).sum();
        let avg_tokens = total_tokens as f64 / snapshots.len() as f64;

        let avg_parser_agreement =
            snapshots.iter().map(|s| s.parser_agreement).sum::<f64>() / snapshots.len() as f64;

        AggregatedMetrics {
            attack_success_rate: attack_successes as f64 / snapshots.len() as f64,
            false_refusal_rate: benign_rejections as f64 / snapshots.len() as f64,
            vault_detection_rate: vault_detections as f64 / snapshots.len() as f64,
            voting_conflict_rate: voting_conflicts as f64 / snapshots.len() as f64,
            policy_enforcement_accuracy: policy_approvals as f64 / snapshots.len() as f64,
            clean_utility: benign_corrects as f64 / snapshots.len() as f64,
            parser_agreement_rate: avg_parser_agreement,
            avg_latency,
            p95_latency,
            p99_latency,
            throughput: snapshots.len() as f64 / avg_latency.as_secs_f64(),
            token_overhead: avg_tokens / 100.0, // Normalized to baseline
        }
    }

    /// Finalize all phases and generate overall metrics
    pub fn finalize(&mut self) {
        // Aggregate all snapshots for overall metrics
        let overall_metrics = self.aggregate_snapshots(&self.dashboard.snapshots);
        self.dashboard.set_overall_metrics(overall_metrics);
    }

    /// Get dashboard
    pub fn dashboard(&self) -> &MetricsDashboard {
        &self.dashboard
    }

    /// Get mutable dashboard
    pub fn dashboard_mut(&mut self) -> &mut MetricsDashboard {
        &mut self.dashboard
    }

    /// Get phase result
    pub fn phase_result(&self, phase_id: &str) -> Option<&Vec<MetricsSnapshot>> {
        self.phase_results.get(phase_id)
    }

    /// Generate execution summary
    pub fn summary(&self) -> ExecutionSummary {
        let enabled_phases: Vec<_> = self
            .phases
            .values()
            .filter(|p| p.enabled)
            .map(|p| p.phase_name.clone())
            .collect();

        ExecutionSummary {
            run_id: self.dashboard.run_id.clone(),
            total_phases: self.phases.len(),
            enabled_phases: enabled_phases.len(),
            total_tests: self.total_tests,
            executed_tests: self.dashboard.snapshots.len(),
            dashboard_summary: self.dashboard.summary(),
        }
    }
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution summary for a test run
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub run_id: String,
    pub total_phases: usize,
    pub enabled_phases: usize,
    pub total_tests: usize,
    pub executed_tests: usize,
    pub dashboard_summary: super::dashboard::DashboardSummary,
}

impl ExecutionSummary {
    pub fn print(&self) {
        println!("\n╔════════════════════════════════════════════════════╗");
        println!("║      Red Team Benchmark Execution Summary          ║");
        println!("╚════════════════════════════════════════════════════╝");
        println!("Run ID: {}", self.run_id);
        println!(
            "Phases: {}/{} enabled",
            self.enabled_phases, self.total_phases
        );
        println!(
            "Tests: {}/{}",
            self.executed_tests, self.total_tests
        );
        println!("\nMetrics:");
        println!(
            "  ASR: {:.2}%",
            self.dashboard_summary.overall_asr * 100.0
        );
        println!(
            "  FRR: {:.2}%",
            self.dashboard_summary.overall_frr * 100.0
        );
        println!(
            "  Vault Detection: {:.2}%",
            self.dashboard_summary.vault_detection_rate * 100.0
        );
        println!(
            "  Parser Agreement: {:.2}%",
            self.dashboard_summary.parser_agreement * 100.0
        );
        println!();
    }
}

/// Test result aggregate
#[derive(Debug, Clone)]
pub struct TestResultAggregate {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub success_rate: f64,
}

impl TestResultAggregate {
    pub fn new(total: usize, passed: usize) -> Self {
        let failed = total - passed;
        let success_rate = if total == 0 {
            0.0
        } else {
            passed as f64 / total as f64
        };

        Self {
            total,
            passed,
            failed,
            success_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let runner = BenchmarkRunner::new();
        assert!(!runner.dashboard.run_id.is_empty());
    }

    #[test]
    fn test_phase_registration() {
        let mut runner = BenchmarkRunner::new();
        let config = PhaseConfig::new(
            "phase_1".to_string(),
            "Phase 1: Direct Injection".to_string(),
            100,
        );
        runner.register_phase(config);

        assert_eq!(runner.phases.len(), 1);
        assert_eq!(runner.total_tests, 100);
    }

    #[test]
    fn test_snapshot_collection() {
        let mut runner = BenchmarkRunner::new();
        let snapshot = MetricsSnapshot::new();
        runner.add_snapshot("phase_1", snapshot);

        assert_eq!(runner.dashboard.snapshots.len(), 1);
    }

    #[test]
    fn test_aggregate_snapshots() {
        let runner = BenchmarkRunner::new();
        let snapshots = vec![
            MetricsSnapshot::new().with_attack_succeeded(true),
            MetricsSnapshot::new().with_attack_succeeded(false),
        ];

        let metrics = runner.aggregate_snapshots(&snapshots);
        assert_eq!(metrics.attack_success_rate, 0.5);
    }

    #[test]
    fn test_execution_summary() {
        let mut runner = BenchmarkRunner::new();
        let config = PhaseConfig::new(
            "phase_1".to_string(),
            "Phase 1".to_string(),
            50,
        );
        runner.register_phase(config);

        let summary = runner.summary();
        assert_eq!(summary.total_tests, 50);
    }
}
