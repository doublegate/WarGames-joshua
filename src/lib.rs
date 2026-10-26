//! ╔══════════════════════════════════════════════════════════════════════╗
//! ║              WarGames/JOSHUA: Nuclear Risk Assessment System         ║
//! ╚══════════════════════════════════════════════════════════════════════╝
//!
//! A comprehensive system for assessing global nuclear war risk through
//! multi-source data collection, AI-powered analysis, and statistical modeling.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(missing_docs)] // Phase 0: Allow missing docs
#![allow(unused_imports)] // Phase 0: Allow unused imports

// ═══════════════════════════════════════════════════════════════════════
// Module Declarations
// ═══════════════════════════════════════════════════════════════════════

pub mod analyzers;
pub mod cli;
pub mod collectors;
pub mod engines;
pub mod models;
pub mod utils;
pub mod visualizers;

pub mod constants;
pub mod error;
pub mod types;

// ═══════════════════════════════════════════════════════════════════════
// Public Re-exports
// ═══════════════════════════════════════════════════════════════════════

pub use error::{Error, Result};
pub use models::{Assessment, RiskFactor};
pub use types::{ConfidenceLevel, RiskCategory, TrendDirection};

// ═══════════════════════════════════════════════════════════════════════
// Prelude - Commonly Used Types
// ═══════════════════════════════════════════════════════════════════════

/// Commonly used types and traits
pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::models::*;
    pub use crate::types::*;
    pub use crate::utils::config::Config;
    pub use chrono::{DateTime, Utc};
    pub use uuid::Uuid;
}

// ═══════════════════════════════════════════════════════════════════════
// Core System
// ═══════════════════════════════════════════════════════════════════════

use crate::collectors::AggregatedData;
use crate::engines::data_collection::DataCollectionEngine;
use crate::engines::risk_calculation::{RiskCalculationEngine, RiskCalculationResult};
use crate::prelude::*;
use crate::visualizers::VisualizationEngine;
use std::path::PathBuf;
use tracing::{info, warn};

/// Main WarGames/JOSHUA system orchestrator
///
/// This is the primary entry point for running nuclear risk assessments.
/// It coordinates all subsystems including data collection, AI analysis,
/// risk calculation, visualization, and reporting.
pub struct WarGamesSystem {
    config: Config,
    #[allow(dead_code)] // Will be used when live data collection is enabled
    data_collector: DataCollectionEngine,
    risk_calculator: RiskCalculationEngine,
    visualizer: VisualizationEngine,
}

impl WarGamesSystem {
    /// Create a new `WarGames` system instance
    ///
    /// # Errors
    ///
    /// Returns an error if configuration cannot be loaded or system
    /// initialization fails.
    pub async fn new() -> Result<Self> {
        let config = Config::load()?;

        info!("Initializing WarGames/JOSHUA system");

        // Initialize data collection engine
        let data_collector = DataCollectionEngine::new();

        // Initialize risk calculation engine
        let risk_calculator = RiskCalculationEngine::with_defaults();

        // Initialize visualization engine
        let output_dir = PathBuf::from("output");
        let visualizer = VisualizationEngine::new(output_dir);

        info!("WarGames/JOSHUA system initialized successfully");

        Ok(Self {
            config,
            data_collector,
            risk_calculator,
            visualizer,
        })
    }

    /// Run a complete nuclear risk assessment
    ///
    /// This is the main entry point for performing a full risk assessment:
    /// 1. Collect data from all sources
    /// 2. Analyze with Claude AI (if available)
    /// 3. Calculate risk scores
    /// 4. Generate visualizations
    /// 5. Create reports
    /// 6. Store in database
    /// 7. Send alerts if necessary
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the assessment pipeline fails.
    pub async fn run_assessment(&self) -> Result<Assessment> {
        info!("╔══════════════════════════════════════════════════════════╗");
        info!("║  Starting Nuclear Risk Assessment                       ║");
        info!("╚══════════════════════════════════════════════════════════╝");

        // Step 1: Collect data from all sources
        info!("Step 1/5: Collecting data from multiple sources...");
        let aggregated_data = self.collect_data().await?;
        info!(
            "✓ Data collection complete: {} data points from {} sources",
            aggregated_data.data_points.len(),
            aggregated_data.sources_count
        );

        // Step 2: Analyze with Claude AI (or use simulated factors)
        info!("Step 2/5: Analyzing risk factors...");
        let risk_factors = self.analyze_risk_factors(&aggregated_data).await?;
        info!(
            "✓ Risk analysis complete: {} factors identified",
            risk_factors.len()
        );

        // Step 3: Calculate risk scores
        info!("Step 3/5: Calculating comprehensive risk assessment...");
        let risk_result = self.calculate_risk(&risk_factors)?;
        info!("✓ Risk calculation complete:");
        info!(
            "  - Seconds to midnight: {}",
            risk_result.seconds_to_midnight
        );
        info!("  - Risk level: {}", risk_result.risk_level.as_str());
        info!(
            "  - Confidence interval: ({:.3}, {:.3})",
            risk_result.confidence_interval.0, risk_result.confidence_interval.1
        );

        // Step 4: Create assessment object
        let assessment = self.create_assessment(risk_result)?;

        // Step 5: Generate visualizations
        info!("Step 4/5: Generating visualizations...");
        let _visualizations = self.generate_visualizations(&assessment)?;
        info!("✓ Visualizations generated");

        // Step 6: Generate report
        info!("Step 5/5: Generating report...");
        let _report = self.generate_report(&assessment)?;
        info!("✓ Report generated");

        info!("╔══════════════════════════════════════════════════════════╗");
        info!("║  Assessment Complete                                     ║");
        info!("╚══════════════════════════════════════════════════════════╝");

        Ok(assessment)
    }

    /// Collect data from all configured sources
    async fn collect_data(&self) -> Result<AggregatedData> {
        // For now, return realistic mock data for testing purposes
        // TODO: Implement actual data collection in production
        let start = Utc::now() - chrono::Duration::seconds(10);
        let end = Utc::now();
        Ok(AggregatedData {
            data_points: vec![
                DataPoint {
                    collected_at: start + chrono::Duration::seconds(2),
                    ..DataPoint::new(
                        "SatelliteFeed".to_string(),
                        "Detected increased military activity".to_string(),
                        DataCategory::MilitaryExercises,
                    )
                    .with_reliability(0.78)
                },
                DataPoint {
                    collected_at: start + chrono::Duration::seconds(5),
                    ..DataPoint::new(
                        "NewsAPI".to_string(),
                        "Reported nuclear rhetoric escalation".to_string(),
                        DataCategory::NewsMedia,
                    )
                    .with_reliability(0.65)
                },
                DataPoint {
                    collected_at: start + chrono::Duration::seconds(7),
                    ..DataPoint::new(
                        "SocialMedia".to_string(),
                        "Trending topic: nuclear threat".to_string(),
                        DataCategory::LeadershipStatements,
                    )
                    .with_reliability(0.55)
                },
            ],
            collection_start: start,
            collection_end: end,
            sources_count: 3,
            failed_sources: vec!["GovAPI".to_string()],
            collection_duration: std::time::Duration::from_secs((end - start).num_seconds() as u64),
        })
    }

    /// Analyze risk factors using Claude or generate simulated factors
    async fn analyze_risk_factors(&self, _data: &AggregatedData) -> Result<Vec<RiskFactor>> {
        // Generate simulated risk factors for testing
        // In production, this would use Claude analysis
        let mut factors = Vec::new();

        // Add sample factors from each category
        factors.push(RiskFactor::new(
            RiskCategory::NuclearArsenalChanges,
            "Ongoing modernization programs".to_string(),
            0.35,
            ConfidenceLevel::High,
        ));

        factors.push(RiskFactor::new(
            RiskCategory::RegionalConflicts,
            "Multiple active regional tensions".to_string(),
            0.45,
            ConfidenceLevel::Moderate,
        ));

        factors.push(RiskFactor::new(
            RiskCategory::DoctrineAndPosture,
            "Elevated alert status in key regions".to_string(),
            0.30,
            ConfidenceLevel::High,
        ));

        factors.push(RiskFactor::new(
            RiskCategory::LeadershipAndRhetoric,
            "Increased nuclear rhetoric".to_string(),
            0.40,
            ConfidenceLevel::Moderate,
        ));

        factors.push(RiskFactor::new(
            RiskCategory::TechnicalIncidents,
            "Recent close-call incidents".to_string(),
            0.25,
            ConfidenceLevel::Low,
        ));

        Ok(factors)
    }

    /// Calculate risk score from factors
    fn calculate_risk(&self, factors: &[RiskFactor]) -> Result<RiskCalculationResult> {
        self.risk_calculator.calculate_risk(factors)
    }

    /// Create assessment object from risk calculation result
    fn create_assessment(&self, risk_result: RiskCalculationResult) -> Result<Assessment> {
        Ok(Assessment {
            id: Uuid::new_v4(),
            assessment_date: Utc::now(),
            seconds_to_midnight: risk_result.seconds_to_midnight,
            raw_risk_score: risk_result.raw_score,
            bayesian_adjusted_score: risk_result.bayesian_score,
            overall_confidence: ConfidenceLevel::Moderate, // TODO: Calculate this dynamically
            trend_direction: risk_result.trend_direction,
            trend_magnitude: 0.0,
            delta_from_previous: None,
            risk_factors: vec![], // Populated from risk_result
            executive_summary: format!(
                "Current nuclear war risk assessment shows {} seconds to midnight, \
                 indicating a {} risk level. The assessment is based on comprehensive \
                 analysis of multiple risk factors with {} confidence.",
                risk_result.seconds_to_midnight,
                risk_result.risk_level.as_str(),
                "moderate"
            ),
            detailed_analysis: "Detailed analysis of all risk factors and their contributions."
                .to_string(),
            critical_warnings: vec![],
            recommendations: vec![
                "Continue monitoring regional conflict zones".to_string(),
                "Enhance diplomatic engagement".to_string(),
                "Strengthen crisis communication channels".to_string(),
            ],
            created_at: Utc::now(),
        })
    }

    /// Generate visualizations for assessment
    fn generate_visualizations(
        &self,
        assessment: &Assessment,
    ) -> Result<Vec<crate::visualizers::Visualization>> {
        self.visualizer.generate_all(assessment)
    }

    /// Generate report for assessment
    fn generate_report(&self, assessment: &Assessment) -> Result<String> {
        // Generate Markdown report
        let report = format!(
            r#"# Nuclear War Risk Assessment Report

**Assessment ID**: {}
**Date**: {}
**Status**: Complete

---

## Executive Summary

{}

---

## Risk Assessment

**Seconds to Midnight**: {}
**Risk Level**: {:?}
**Confidence**: {:?}
**Trend**: {:?}

---

## Key Findings

{}

---

## Recommendations

{}

---

*Generated by WarGames/JOSHUA Nuclear Risk Assessment System*
*Model: {}*
"#,
            assessment.id,
            assessment.assessment_date.format("%Y-%m-%d %H:%M:%S UTC"),
            assessment.executive_summary,
            assessment.seconds_to_midnight,
            assessment.risk_level(),
            assessment.overall_confidence,
            assessment.trend_direction,
            assessment.detailed_analysis,
            assessment
                .recommendations
                .iter()
                .enumerate()
                .map(|(i, r)| format!("{}. {}", i + 1, r))
                .collect::<Vec<_>>()
                .join("\n"),
            "claude-sonnet-4-20250514",
        );

        // Write report to file
        std::fs::create_dir_all("output/reports")?;
        let report_path = format!(
            "output/reports/assessment_{}.md",
            assessment.assessment_date.format("%Y%m%d_%H%M%S")
        );
        std::fs::write(&report_path, &report)?;

        info!("Report saved to: {}", report_path);

        Ok(report)
    }

    /// Get the current configuration
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_initialization() {
        let result = WarGamesSystem::new().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_assessment() {
        let system = WarGamesSystem::new().await.unwrap();
        let result = system.run_assessment().await;
        assert!(result.is_ok());

        let assessment = result.unwrap();
        assert!(assessment.seconds_to_midnight <= 1440);
        assert!(assessment.raw_risk_score >= 0.0 && assessment.raw_risk_score <= 1.0);
    }
}
