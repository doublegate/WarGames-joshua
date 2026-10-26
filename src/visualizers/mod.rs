//! Visualization generators for risk assessments.

use crate::prelude::*;
use plotters::prelude::*;
use std::path::PathBuf;

/// Trait for visualization generation
pub trait Visualizer: Send + Sync {
    /// Generate visualization from assessment data
    fn visualize(&self, assessment: &Assessment) -> Result<Visualization>;

    /// Name of this visualization
    fn name(&self) -> &str;

    /// Output format(s) supported
    fn supported_formats(&self) -> &[VisualizationFormat];

    /// Default output format
    fn default_format(&self) -> VisualizationFormat {
        VisualizationFormat::Svg
    }
}

/// Generated visualization
#[derive(Debug, Clone)]
pub struct Visualization {
    /// Visualization name
    pub name: String,

    /// Output format
    pub format: VisualizationFormat,

    /// File path where visualization is saved
    pub file_path: PathBuf,

    /// Metadata about the visualization
    pub metadata: VisualizationMetadata,
}

/// Metadata about a visualization
#[derive(Debug, Clone)]
pub struct VisualizationMetadata {
    /// Width in pixels
    pub width: u32,

    /// Height in pixels
    pub height: u32,

    /// When generated
    pub generated_at: DateTime<Utc>,
}

/// Doomsday Clock visualizer
pub struct DoomsdayClockVisualizer {
    output_dir: PathBuf,
}

impl DoomsdayClockVisualizer {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    fn draw_clock(&self, seconds: u32, file_path: &PathBuf) -> Result<()> {
        let root = SVGBackend::new(file_path, (800, 800)).into_drawing_area();
        root.fill(&RGBColor(245, 245, 245))
            .map_err(|e| Error::Visualization(format!("Failed to fill background: {}", e)))?;

        let clock_center = (400.0, 400.0);
        let clock_radius = 320.0;

        // Draw clock face outer circle
        root.draw(&Circle::new(
            (clock_center.0 as i32, clock_center.1 as i32),
            clock_radius as i32,
            BLACK.mix(0.2).filled(),
        ))
        .map_err(|e| Error::Visualization(format!("Failed to draw clock face: {}", e)))?;

        // Draw inner circle
        root.draw(&Circle::new(
            (clock_center.0 as i32, clock_center.1 as i32),
            (clock_radius - 20.0) as i32,
            RGBColor(245, 245, 245).filled(),
        ))
        .map_err(|e| Error::Visualization(format!("Failed to draw inner circle: {}", e)))?;

        // Draw hour markers
        for i in 0..12 {
            let angle = (i as f64 * 30.0 - 90.0).to_radians();
            let x1 = clock_center.0 + (clock_radius - 40.0) * angle.cos();
            let y1 = clock_center.1 + (clock_radius - 40.0) * angle.sin();
            let x2 = clock_center.0 + (clock_radius - 20.0) * angle.cos();
            let y2 = clock_center.1 + (clock_radius - 20.0) * angle.sin();

            root.draw(&PathElement::new(
                vec![(x1 as i32, y1 as i32), (x2 as i32, y2 as i32)],
                BLACK.stroke_width(3),
            ))
            .map_err(|e| Error::Visualization(format!("Failed to draw markers: {}", e)))?;
        }

        // Calculate minute hand angle (pointing to midnight)
        let minutes_to_midnight = seconds as f64 / 60.0;
        let minute_angle = (90.0 - minutes_to_midnight * 6.0).to_radians(); // 6 degrees per minute

        // Draw minute hand
        let hand_length = clock_radius - 60.0;
        let hand_x = clock_center.0 + hand_length * minute_angle.cos();
        let hand_y = clock_center.1 - hand_length * minute_angle.sin();

        root.draw(&PathElement::new(
            vec![
                (clock_center.0 as i32, clock_center.1 as i32),
                (hand_x as i32, hand_y as i32),
            ],
            RED.stroke_width(6),
        ))
        .map_err(|e| Error::Visualization(format!("Failed to draw minute hand: {}", e)))?;

        // Draw center dot
        root.draw(&Circle::new(
            (clock_center.0 as i32, clock_center.1 as i32),
            12,
            RED.filled(),
        ))
        .map_err(|e| Error::Visualization(format!("Failed to draw center: {}", e)))?;

        // Draw "MIDNIGHT" text at top
        root.draw(&Text::new(
            "MIDNIGHT",
            (clock_center.0 as i32, 60),
            ("sans-serif", 28, &BLACK).into_text_style(&root),
        ))
        .map_err(|e| Error::Visualization(format!("Failed to draw text: {}", e)))?;

        // Draw seconds count
        root.draw(&Text::new(
            format!("{} seconds to midnight", seconds),
            (clock_center.0 as i32, 720),
            ("sans-serif", 24, &RED).into_text_style(&root),
        ))
        .map_err(|e| Error::Visualization(format!("Failed to draw seconds text: {}", e)))?;

        // Determine risk level color
        let risk_color = if seconds < 100 {
            RGBColor(139, 0, 0) // Dark red - Critical
        } else if seconds < 200 {
            RGBColor(220, 20, 60) // Crimson - Severe
        } else if seconds < 400 {
            RGBColor(255, 69, 0) // Red-Orange - High
        } else if seconds < 600 {
            RGBColor(255, 140, 0) // Dark Orange - Moderate
        } else if seconds < 900 {
            RGBColor(255, 215, 0) // Gold - Low
        } else {
            RGBColor(50, 205, 50) // Lime Green - Minimal
        };

        let risk_text = if seconds < 100 {
            "CRITICAL"
        } else if seconds < 200 {
            "SEVERE"
        } else if seconds < 400 {
            "HIGH"
        } else if seconds < 600 {
            "MODERATE"
        } else if seconds < 900 {
            "LOW"
        } else {
            "MINIMAL"
        };

        root.draw(&Text::new(
            format!("Risk Level: {}", risk_text),
            (clock_center.0 as i32, 760),
            ("sans-serif", 20, &risk_color).into_text_style(&root),
        ))
        .map_err(|e| Error::Visualization(format!("Failed to draw risk level: {}", e)))?;

        root.present()
            .map_err(|e| Error::Visualization(format!("Failed to present: {}", e)))?;

        Ok(())
    }
}

impl Visualizer for DoomsdayClockVisualizer {
    fn visualize(&self, assessment: &Assessment) -> Result<Visualization> {
        std::fs::create_dir_all(&self.output_dir)?;

        let file_name = format!("doomsday_clock_{}.svg", Utc::now().format("%Y%m%d_%H%M%S"));
        let file_path = self.output_dir.join(&file_name);

        self.draw_clock(assessment.seconds_to_midnight, &file_path)?;

        Ok(Visualization {
            name: self.name().to_string(),
            format: VisualizationFormat::Svg,
            file_path,
            metadata: VisualizationMetadata {
                width: 800,
                height: 800,
                generated_at: Utc::now(),
            },
        })
    }

    fn name(&self) -> &str {
        "Doomsday Clock"
    }

    fn supported_formats(&self) -> &[VisualizationFormat] {
        &[VisualizationFormat::Svg]
    }
}

/// Visualization engine that manages multiple visualizers
pub struct VisualizationEngine {
    output_dir: PathBuf,
}

impl VisualizationEngine {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    pub fn generate_all(&self, assessment: &Assessment) -> Result<Vec<Visualization>> {
        let mut visualizations = Vec::new();

        // Generate doomsday clock
        let clock_viz = DoomsdayClockVisualizer::new(self.output_dir.clone());
        visualizations.push(clock_viz.visualize(assessment)?);

        Ok(visualizations)
    }
}
