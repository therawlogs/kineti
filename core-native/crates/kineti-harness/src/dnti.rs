//! Directional Normalized Trust-Weighted Impact (DNTI) Metric Engine.
//!
//! Implements the three-factor outcome verification metric defined in Paper 5 (§3.1):
//! $$\text{Verified Impact} = \Phi(\text{Metric}_{\text{base}}, \text{Metric}_{\text{obs}}, \mathbf{d}) \times \sigma_{\tau}(SE) \times \Psi(\mathcal{T})$$
//!
//! Author: Praveen Kumar (therawlogs.com | Foundational AI Research)

/// Direction of improvement for an evaluated metric.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricDirection {
    /// Lower observed value is better (e.g., latency, defect count, MTTR, memory usage).
    Minimization,
    /// Higher observed value is better (e.g., throughput, test coverage, success rate).
    Maximization,
}

impl MetricDirection {
    /// Returns the directional scalar $d$: $+1.0$ for minimization, $-1.0$ for maximization.
    pub fn direction_scalar(&self) -> f64 {
        match self {
            Self::Minimization => 1.0,
            Self::Maximization => -1.0,
        }
    }
}

/// Default temperature attenuation factor $\tau = 0.15$ per Paper 5.
pub const DEFAULT_TAU: f64 = 0.15;

/// Default minimum assertions required for test proof integrity.
pub const DEFAULT_MIN_ASSERTIONS: u32 = 1;

/// Default minimum diff coverage percentage required (80%).
pub const DEFAULT_MIN_COVERAGE_PCT: f64 = 80.0;

/// Default numerical stability epsilon.
pub const DEFAULT_EPSILON: f64 = 1e-6;

/// Calculates the Directional Normalized Delta $\Phi(\text{Metric}_{\text{base}}, \text{Metric}_{\text{obs}}, \mathbf{d})$.
///
/// $$\Phi = \mathbf{d} \cdot \left( \frac{\text{Metric}_{\text{base}} - \text{Metric}_{\text{obs}}}{\max(|\text{Metric}_{\text{base}} - \text{Metric}_{\text{target}}|, \epsilon)} \right)$$
pub fn calculate_phi(
    base: f64,
    obs: f64,
    target: f64,
    direction: MetricDirection,
    epsilon: f64,
) -> f64 {
    let d = direction.direction_scalar();
    let denom = (base - target).abs().max(epsilon.max(1e-12));
    d * ((base - obs) / denom)
}

/// Computes Bounded Exponential Semantic Entropy Attenuation $\sigma_\tau(SE) \in (0, 1]$.
///
/// $$\sigma_\tau(SE) = \exp\left(-\frac{SE}{\tau}\right)$$
pub fn semantic_entropy_attenuation(se: f64, tau: f64) -> f64 {
    let clamped_se = se.max(0.0);
    let effective_tau = tau.max(1e-6);
    (-clamped_se / effective_tau).exp()
}

/// Multi-Point Test Integrity Predicate $\Psi(\mathcal{T})$.
///
/// $$\Psi(\mathcal{T}) = \mathbb{I}(\text{ExitCode} == 0) \times \mathbb{I}(\text{Assertions} \ge K_{\min}) \times \mathbb{I}(\text{DiffCoverage} \ge \theta_{\text{cov}}) \times (1 - \text{TamperFlag})$$
#[derive(Clone, Debug, PartialEq)]
pub struct TestIntegrityPredicate {
    /// Process exit code (0 for success).
    pub exit_code: i32,
    /// Number of verified assertions passed.
    pub assertions_passed: u32,
    /// Minimum assertions threshold $K_{\min}$.
    pub min_assertions: u32,
    /// Evaluated diff test coverage percentage (0.0 to 100.0).
    pub diff_coverage_pct: f64,
    /// Minimum required test coverage percentage $\theta_{\text{cov}}$.
    pub min_coverage_pct: f64,
    /// Whether journal or artifact tampering was detected.
    pub tamper_detected: bool,
}

impl Default for TestIntegrityPredicate {
    fn default() -> Self {
        Self {
            exit_code: 0,
            assertions_passed: 1,
            min_assertions: DEFAULT_MIN_ASSERTIONS,
            diff_coverage_pct: 100.0,
            min_coverage_pct: DEFAULT_MIN_COVERAGE_PCT,
            tamper_detected: false,
        }
    }
}

impl TestIntegrityPredicate {
    /// Evaluates the multi-point predicate, returning 1.0 if all integrity checks pass, or 0.0 otherwise.
    pub fn evaluate(&self) -> f64 {
        let pass_exit = if self.exit_code == 0 { 1.0 } else { 0.0 };
        let pass_assert = if self.assertions_passed >= self.min_assertions {
            1.0
        } else {
            0.0
        };
        let pass_coverage = if self.diff_coverage_pct >= self.min_coverage_pct {
            1.0
        } else {
            0.0
        };
        let pass_tamper = if self.tamper_detected { 0.0 } else { 1.0 };

        pass_exit * pass_assert * pass_coverage * pass_tamper
    }
}

/// Direct DNTI calculation from constituent components.
///
/// $$\text{Verified Impact} = \Phi \times \sigma_\tau(SE) \times \Psi(\mathcal{T})$$
pub fn calculate_dnti(phi: f64, se_attenuation: f64, predicate: &TestIntegrityPredicate) -> f64 {
    phi * se_attenuation * predicate.evaluate()
}

/// Comprehensive input specification for DNTI evaluation.
#[derive(Clone, Debug, PartialEq)]
pub struct DntiInput {
    /// Evaluated metric name.
    pub metric_name: String,
    /// Baseline metric value before agent intervention.
    pub metric_base: f64,
    /// Observed metric value after agent intervention.
    pub metric_obs: f64,
    /// Target goal metric value.
    pub metric_target: f64,
    /// Optimization direction (minimization vs maximization).
    pub direction: MetricDirection,
    /// Measured semantic entropy (SE >= 0.0).
    pub semantic_entropy: f64,
    /// Optional temperature attenuation parameter (defaults to 0.15).
    pub tau: Option<f64>,
    /// Test integrity predicate evaluation parameters.
    pub predicate: TestIntegrityPredicate,
}

/// Detailed result of a DNTI evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DntiResult {
    /// Directional normalized delta $\Phi$.
    pub directional_delta: f64,
    /// Semantic entropy attenuation $\sigma_\tau(SE)$.
    pub semantic_entropy_attenuation: f64,
    /// Test integrity predicate evaluation $\Psi(\mathcal{T}) \in \{0.0, 1.0\}$.
    pub test_integrity_predicate: f64,
    /// Final verified impact score.
    pub verified_impact: f64,
    /// Whether the outcome meets passing verification criteria.
    pub passed: bool,
}

/// Evaluates DNTI across all three mathematical factors.
pub fn compute_dnti(input: &DntiInput) -> DntiResult {
    let tau = input.tau.unwrap_or(DEFAULT_TAU);
    let phi = calculate_phi(
        input.metric_base,
        input.metric_obs,
        input.metric_target,
        input.direction,
        DEFAULT_EPSILON,
    );
    let se_att = semantic_entropy_attenuation(input.semantic_entropy, tau);
    let psi = input.predicate.evaluate();
    let verified_impact = phi * se_att * psi;
    let passed = psi > 0.0 && verified_impact > 0.0;

    DntiResult {
        directional_delta: phi,
        semantic_entropy_attenuation: se_att,
        test_integrity_predicate: psi,
        verified_impact,
        passed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dnti_latency_minimization() {
        let input = DntiInput {
            metric_name: "p99_latency_ms".into(),
            metric_base: 1240.0,
            metric_obs: 42.0,
            metric_target: 500.0,
            direction: MetricDirection::Minimization,
            semantic_entropy: 0.042,
            tau: Some(0.15),
            predicate: TestIntegrityPredicate {
                exit_code: 0,
                assertions_passed: 14,
                min_assertions: 1,
                diff_coverage_pct: 100.0,
                min_coverage_pct: 80.0,
                tamper_detected: false,
            },
        };

        let res = compute_dnti(&input);
        assert!(res.passed);
        assert_eq!(res.test_integrity_predicate, 1.0);
        assert!(res.directional_delta > 1.0);
        assert!(res.semantic_entropy_attenuation > 0.70);
        assert!(res.verified_impact > 1.0);
    }

    #[test]
    fn test_dnti_tamper_and_test_failure_zeroes_impact() {
        // Failing exit code zeroes impact
        let failed_tests = TestIntegrityPredicate {
            exit_code: 1,
            assertions_passed: 0,
            min_assertions: 1,
            diff_coverage_pct: 50.0,
            min_coverage_pct: 80.0,
            tamper_detected: false,
        };
        assert_eq!(failed_tests.evaluate(), 0.0);

        // Tamper detected zeroes impact
        let tampered = TestIntegrityPredicate {
            exit_code: 0,
            assertions_passed: 10,
            min_assertions: 1,
            diff_coverage_pct: 95.0,
            min_coverage_pct: 80.0,
            tamper_detected: true,
        };
        assert_eq!(tampered.evaluate(), 0.0);
    }

    #[test]
    fn test_dnti_semantic_entropy_attenuation() {
        let low_se = semantic_entropy_attenuation(0.02, DEFAULT_TAU);
        let high_se = semantic_entropy_attenuation(0.60, DEFAULT_TAU);
        assert!(low_se > high_se);
        assert!(low_se > 0.85);
        assert!(high_se < 0.05);
    }
}
