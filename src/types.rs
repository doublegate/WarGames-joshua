//! Type aliases and common types used throughout the WarGames/JOSHUA system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Timestamp type alias
pub type Timestamp = DateTime<Utc>;

/// Assessment ID type alias
pub type AssessmentId = Uuid;

/// Risk score (0.0 to 1.0)
pub type RiskScore = f64;

/// Confidence level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    VeryLow,
    Low,
    Moderate,
    High,
    VeryHigh,
}

impl ConfidenceLevel {
    /// Convert confidence level to numeric score (0.0 to 1.0)
    pub fn to_score(&self) -> f64 {
        match self {
            ConfidenceLevel::VeryLow => 0.2,
            ConfidenceLevel::Low => 0.4,
            ConfidenceLevel::Moderate => 0.6,
            ConfidenceLevel::High => 0.8,
            ConfidenceLevel::VeryHigh => 1.0,
        }
    }

    /// Alias for to_score
    pub fn to_numeric(&self) -> f64 {
        self.to_score()
    }

    /// Convert numeric score to confidence level
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.3 => ConfidenceLevel::VeryLow,
            s if s < 0.5 => ConfidenceLevel::Low,
            s if s < 0.7 => ConfidenceLevel::Moderate,
            s if s < 0.9 => ConfidenceLevel::High,
            _ => ConfidenceLevel::VeryHigh,
        }
    }
}

/// Trend direction enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Deteriorating,
    Stable,
    Uncertain,
}

/// Risk category enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskCategory {
    NuclearArsenalChanges,
    DoctrineAndPosture,
    ArmsControlBreakdown,
    RegionalConflicts,
    LeadershipAndRhetoric,
    LeadershipInstability,
    TechnicalIncidents,
    CommunicationBreakdown,
    CommunicationFailures,
    EmergingTechnology,
    EmergingTechRisks,
    EconomicFactors,
    EconomicPressure,
}

impl RiskCategory {
    /// Get all risk categories
    pub fn all() -> Vec<Self> {
        vec![
            RiskCategory::NuclearArsenalChanges,
            RiskCategory::ArmsControlBreakdown,
            RiskCategory::RegionalConflicts,
            RiskCategory::LeadershipInstability,
            RiskCategory::TechnicalIncidents,
            RiskCategory::CommunicationFailures,
            RiskCategory::EmergingTechRisks,
            RiskCategory::EconomicPressure,
        ]
    }

    /// Get default weight for this category
    pub fn default_weight(&self) -> f64 {
        match self {
            RiskCategory::RegionalConflicts => 0.20,
            RiskCategory::NuclearArsenalChanges
            | RiskCategory::DoctrineAndPosture
            | RiskCategory::ArmsControlBreakdown
            | RiskCategory::TechnicalIncidents => 0.15,
            RiskCategory::LeadershipInstability
            | RiskCategory::LeadershipAndRhetoric
            | RiskCategory::CommunicationBreakdown
            | RiskCategory::CommunicationFailures
            | RiskCategory::EmergingTechnology
            | RiskCategory::EmergingTechRisks => 0.10,
            RiskCategory::EconomicFactors | RiskCategory::EconomicPressure => 0.05,
        }
    }
}

/// Data category for collected information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataCategory {
    NewsMedia,
    NuclearArsenal,
    RegionalConflict,
    DiplomaticRelations,
    MilitaryExercises,
    TreatyCompliance,
    LeadershipStatements,
    TechnicalIncident,
}

/// Impact level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Severe,
    Critical,
    Apocalyptic,
}

/// Visualization format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualizationFormat {
    Svg,
    Png,
    Html,
    Ascii,
}

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Markdown,
    Html,
    Json,
    Pdf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_level_conversion() {
        assert!((ConfidenceLevel::High.to_score() - 0.8).abs() < f64::EPSILON);
        assert_eq!(ConfidenceLevel::from_score(0.95), ConfidenceLevel::VeryHigh);
    }

    #[test]
    fn test_risk_category_weights() {
        let weights: f64 = RiskCategory::all()
            .iter()
            .map(RiskCategory::default_weight)
            .sum();
        assert!((weights - 1.0).abs() < 0.001);
    }
}
