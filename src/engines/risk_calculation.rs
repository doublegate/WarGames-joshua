//! Risk calculation engine implementation.
//!
//! This module implements the core risk calculation algorithms including:
//! - Multi-factor weighted scoring
//! - Bayesian risk adjustment
//! - Monte Carlo simulation
//! - Uncertainty quantification

use crate::prelude::*;
use std::collections::HashMap;

/// Risk calculation engine configuration
#[derive(Debug, Clone)]
pub struct RiskCalculationConfig {
    /// Category weights (must sum to 1.0)
    pub category_weights: HashMap<RiskCategory, f64>,

    /// Monte Carlo iteration count
    pub monte_carlo_iterations: usize,

    /// Bayesian prior strength
    pub bayesian_prior_strength: f64,

    /// Enable/disable features
    pub enable_bayesian_adjustment: bool,
    pub enable_monte_carlo: bool,
}

impl Default for RiskCalculationConfig {
    fn default() -> Self {
        let mut category_weights = HashMap::new();
        category_weights.insert(RiskCategory::NuclearArsenalChanges, 0.15);
        category_weights.insert(RiskCategory::DoctrineAndPosture, 0.15);
        category_weights.insert(RiskCategory::RegionalConflicts, 0.20);
        category_weights.insert(RiskCategory::LeadershipAndRhetoric, 0.10);
        category_weights.insert(RiskCategory::TechnicalIncidents, 0.15);
        category_weights.insert(RiskCategory::CommunicationBreakdown, 0.10);
        category_weights.insert(RiskCategory::EmergingTechnology, 0.10);
        category_weights.insert(RiskCategory::EconomicFactors, 0.05);

        Self {
            category_weights,
            monte_carlo_iterations: 10_000,
            bayesian_prior_strength: 0.3,
            enable_bayesian_adjustment: true,
            enable_monte_carlo: true,
        }
    }
}

/// Risk calculation result with comprehensive statistics
#[derive(Debug, Clone)]
pub struct RiskCalculationResult {
    /// Raw weighted score [0.0, 1.0]
    pub raw_score: f64,

    /// Bayesian adjusted score [0.0, 1.0]
    pub bayesian_score: f64,

    /// Seconds to midnight (0-1440, where 0 is midnight)
    pub seconds_to_midnight: u32,

    /// Confidence interval (lower, upper)
    pub confidence_interval: (f64, f64),

    /// Risk level categorization
    pub risk_level: RiskLevel,

    /// Trend direction
    pub trend_direction: TrendDirection,

    /// Primary risk drivers (top contributors)
    pub primary_drivers: Vec<(String, f64)>,

    /// Category breakdowns
    pub category_scores: HashMap<RiskCategory, f64>,

    /// Monte Carlo statistics
    pub monte_carlo_stats: Option<MonteCarloStatistics>,
}

/// Monte Carlo simulation statistics
#[derive(Debug, Clone)]
pub struct MonteCarloStatistics {
    pub mean: f64,
    pub std_dev: f64,
    pub percentile_5: f64,
    pub percentile_95: f64,
    pub median: f64,
}

/// Risk level categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Minimal,  // >900 seconds
    Low,      // 600-900 seconds
    Moderate, // 400-600 seconds
    High,     // 200-400 seconds
    Severe,   // 100-200 seconds
    Critical, // <100 seconds
}

impl RiskLevel {
    pub fn from_seconds(seconds: u32) -> Self {
        match seconds {
            0..=100 => Self::Critical,
            101..=200 => Self::Severe,
            201..=400 => Self::High,
            401..=600 => Self::Moderate,
            601..=900 => Self::Low,
            _ => Self::Minimal,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "CRITICAL",
            Self::Severe => "SEVERE",
            Self::High => "HIGH",
            Self::Moderate => "MODERATE",
            Self::Low => "LOW",
            Self::Minimal => "MINIMAL",
        }
    }
}

/// Risk calculation engine
pub struct RiskCalculationEngine {
    config: RiskCalculationConfig,
    historical_baseline: f64,
}

impl RiskCalculationEngine {
    /// Create a new risk calculation engine
    pub fn new(config: RiskCalculationConfig) -> Self {
        Self {
            config,
            historical_baseline: 0.062, // 89 seconds baseline (89/1440)
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(RiskCalculationConfig::default())
    }

    /// Calculate comprehensive risk assessment
    pub fn calculate_risk(&self, factors: &[RiskFactor]) -> Result<RiskCalculationResult> {
        if factors.is_empty() {
            return Err(Error::Validation(
                "Cannot calculate risk with no factors".to_string(),
            ));
        }

        // Step 1: Calculate category scores
        let category_scores = self.calculate_category_scores(factors)?;

        // Step 2: Calculate raw weighted score
        let raw_score = self.calculate_weighted_score(&category_scores)?;

        // Step 3: Apply Bayesian adjustment
        let bayesian_score = if self.config.enable_bayesian_adjustment {
            self.apply_bayesian_adjustment(raw_score, factors)?
        } else {
            raw_score
        };

        // Step 4: Run Monte Carlo simulation for confidence intervals
        let (confidence_interval, monte_carlo_stats) = if self.config.enable_monte_carlo {
            let stats = self.run_monte_carlo_simulation(factors)?;
            let ci = (stats.percentile_5, stats.percentile_95);
            (ci, Some(stats))
        } else {
            ((bayesian_score * 0.9, bayesian_score * 1.1), None)
        };

        // Step 5: Convert to seconds to midnight
        let seconds_to_midnight = self.score_to_seconds(bayesian_score);

        // Step 6: Categorize risk level
        let risk_level = RiskLevel::from_seconds(seconds_to_midnight);

        // Step 7: Determine trend direction
        let trend_direction = self.determine_trend(factors);

        // Step 8: Identify primary drivers
        let primary_drivers = self.identify_primary_drivers(factors, &category_scores)?;

        Ok(RiskCalculationResult {
            raw_score,
            bayesian_score,
            seconds_to_midnight,
            confidence_interval,
            risk_level,
            trend_direction,
            primary_drivers,
            category_scores,
            monte_carlo_stats,
        })
    }

    /// Calculate scores by category
    fn calculate_category_scores(
        &self,
        factors: &[RiskFactor],
    ) -> Result<HashMap<RiskCategory, f64>> {
        let mut category_scores: HashMap<RiskCategory, Vec<f64>> = HashMap::new();

        // Group factors by category
        for factor in factors {
            category_scores
                .entry(factor.category)
                .or_insert_with(Vec::new)
                .push(factor.value);
        }

        // Calculate average score per category
        let mut result = HashMap::new();
        for (category, values) in category_scores {
            if !values.is_empty() {
                let avg = values.iter().sum::<f64>() / values.len() as f64;
                result.insert(category, avg.clamp(0.0, 1.0));
            }
        }

        Ok(result)
    }

    /// Calculate weighted score from category scores
    fn calculate_weighted_score(
        &self,
        category_scores: &HashMap<RiskCategory, f64>,
    ) -> Result<f64> {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (category, score) in category_scores {
            if let Some(&weight) = self.config.category_weights.get(category) {
                weighted_sum += score * weight;
                total_weight += weight;
            }
        }

        // Normalize if we don't have all categories
        let final_score = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };

        Ok(final_score.clamp(0.0, 1.0))
    }

    /// Apply Bayesian adjustment based on historical priors
    fn apply_bayesian_adjustment(&self, raw_score: f64, factors: &[RiskFactor]) -> Result<f64> {
        // Calculate average confidence
        let avg_confidence: f64 = factors
            .iter()
            .map(|f| f.confidence.to_numeric())
            .sum::<f64>()
            / factors.len().max(1) as f64;

        // Bayesian update: weighted average of raw score and historical baseline
        // Higher confidence = more weight on raw score
        // Lower confidence = more weight on baseline
        let confidence_weight = avg_confidence;
        let baseline_weight = (1.0 - avg_confidence) * self.config.bayesian_prior_strength;

        let total_weight = confidence_weight + baseline_weight;
        let adjusted = (raw_score * confidence_weight + self.historical_baseline * baseline_weight)
            / total_weight;

        Ok(adjusted.clamp(0.0, 1.0))
    }

    /// Run Monte Carlo simulation for uncertainty quantification
    fn run_monte_carlo_simulation(&self, factors: &[RiskFactor]) -> Result<MonteCarloStatistics> {
        let mut simulated_scores = Vec::with_capacity(self.config.monte_carlo_iterations);

        // Use deterministic variations instead of random sampling
        for i in 0..self.config.monte_carlo_iterations {
            let mut sim_factors = Vec::new();
            let variation = ((i as f64 / self.config.monte_carlo_iterations as f64) - 0.5) * 0.2;

            for factor in factors {
                let uncertainty = 1.0 - factor.confidence.to_numeric();
                let noise = variation * uncertainty;
                let simulated_value = (factor.value + noise).clamp(0.0, 1.0);

                let mut sim_factor = factor.clone();
                sim_factor.value = simulated_value;
                sim_factors.push(sim_factor);
            }

            // Calculate score for this simulation
            let category_scores = self.calculate_category_scores(&sim_factors)?;
            let score = self.calculate_weighted_score(&category_scores)?;
            simulated_scores.push(score);
        }

        // Calculate statistics
        simulated_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mean = simulated_scores.iter().sum::<f64>() / simulated_scores.len() as f64;
        let variance = simulated_scores
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / simulated_scores.len() as f64;
        let std_dev = variance.sqrt();

        let percentile_5 = simulated_scores[(simulated_scores.len() as f64 * 0.05) as usize];
        let percentile_95 = simulated_scores[(simulated_scores.len() as f64 * 0.95) as usize];
        let median = simulated_scores[simulated_scores.len() / 2];

        Ok(MonteCarloStatistics {
            mean,
            std_dev,
            percentile_5,
            percentile_95,
            median,
        })
    }

    /// Convert normalized score [0.0, 1.0] to seconds to midnight [0, 1440]
    fn score_to_seconds(&self, score: f64) -> u32 {
        // Inverse relationship: higher score = fewer seconds to midnight
        // 0.0 score = 1440 seconds (noon, minimal risk)
        // 1.0 score = 0 seconds (midnight, maximum risk)
        let seconds = 1440.0 * (1.0 - score);
        seconds.round() as u32
    }

    /// Determine overall trend direction
    fn determine_trend(&self, factors: &[RiskFactor]) -> TrendDirection {
        let deteriorating = factors
            .iter()
            .filter(|f| matches!(f.trend, Some(TrendDirection::Deteriorating)))
            .count();
        let improving = factors
            .iter()
            .filter(|f| matches!(f.trend, Some(TrendDirection::Improving)))
            .count();

        if deteriorating > improving * 2 {
            TrendDirection::Deteriorating
        } else if improving > deteriorating * 2 {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        }
    }

    /// Identify primary risk drivers
    fn identify_primary_drivers(
        &self,
        factors: &[RiskFactor],
        _category_scores: &HashMap<RiskCategory, f64>,
    ) -> Result<Vec<(String, f64)>> {
        // Calculate contribution of each factor
        let mut contributions: Vec<(String, f64)> = Vec::new();

        for factor in factors {
            if let Some(&cat_weight) = self.config.category_weights.get(&factor.category) {
                // Contribution = factor value * category weight
                let contribution = factor.value * cat_weight;
                contributions.push((factor.name.clone(), contribution));
            }
        }

        // Sort by contribution (descending) and take top 5
        contributions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        contributions.truncate(5);

        Ok(contributions)
    }

    /// Calculate delta from previous assessment
    pub fn calculate_delta(
        &self,
        current: &RiskCalculationResult,
        previous: &RiskCalculationResult,
    ) -> i32 {
        current.seconds_to_midnight as i32 - previous.seconds_to_midnight as i32
    }
}

impl Default for RiskCalculationEngine {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_factor(category: RiskCategory, value: f64) -> RiskFactor {
        RiskFactor::new(
            category,
            "Test Factor".to_string(),
            value,
            ConfidenceLevel::High,
        )
    }

    #[test]
    fn test_risk_calculation_basic() {
        let engine = RiskCalculationEngine::with_defaults();

        let factors = vec![
            create_test_factor(RiskCategory::NuclearArsenalChanges, 0.3),
            create_test_factor(RiskCategory::RegionalConflicts, 0.5),
        ];

        let result = engine.calculate_risk(&factors).unwrap();

        assert!(result.raw_score >= 0.0 && result.raw_score <= 1.0);
        assert!(result.seconds_to_midnight <= 1440);
    }

    #[test]
    fn test_score_to_seconds() {
        let engine = RiskCalculationEngine::with_defaults();

        assert_eq!(engine.score_to_seconds(0.0), 1440); // No risk = noon
        assert_eq!(engine.score_to_seconds(1.0), 0); // Max risk = midnight
        assert_eq!(engine.score_to_seconds(0.5), 720); // 50% risk = 6am
    }

    #[test]
    fn test_risk_level_categorization() {
        assert_eq!(RiskLevel::from_seconds(50), RiskLevel::Critical);
        assert_eq!(RiskLevel::from_seconds(150), RiskLevel::Severe);
        assert_eq!(RiskLevel::from_seconds(300), RiskLevel::High);
        assert_eq!(RiskLevel::from_seconds(500), RiskLevel::Moderate);
        assert_eq!(RiskLevel::from_seconds(750), RiskLevel::Low);
        assert_eq!(RiskLevel::from_seconds(1000), RiskLevel::Minimal);
    }

    #[test]
    fn test_category_weights_sum() {
        let config = RiskCalculationConfig::default();
        let sum: f64 = config.category_weights.values().sum();
        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_empty_factors() {
        let engine = RiskCalculationEngine::with_defaults();
        let result = engine.calculate_risk(&[]);
        assert!(result.is_err());
    }
}
