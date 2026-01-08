//! Scenario definition and RON file loading

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::actions::ScenarioAction;
use super::validated_types::ValidatedMaterialId;
use super::verification::VerificationCondition;

/// Top-level scenario definition loaded from RON files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDefinition {
    /// Scenario name
    pub name: String,

    /// Description
    pub description: String,

    /// Initial setup actions (run before main scenario)
    #[serde(default)]
    pub setup: Vec<ScenarioAction>,

    /// Main scenario actions
    pub actions: Vec<ScenarioAction>,

    /// Verification checks to run after scenario
    #[serde(default)]
    pub verify: Vec<VerificationCondition>,

    /// Cleanup actions (run even if scenario fails)
    #[serde(default)]
    pub cleanup: Vec<ScenarioAction>,
}

impl ScenarioDefinition {
    /// Load scenario from RON file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read scenario file: {}", path.display()))?;

        let scenario = ron::from_str(&content)
            .with_context(|| format!("Failed to parse RON scenario: {}", path.display()))?;

        Ok(scenario)
    }

    /// Save scenario to RON file
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let ron = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .context("Failed to serialize scenario to RON")?;

        std::fs::write(path.as_ref(), ron).with_context(|| {
            format!("Failed to write scenario file: {}", path.as_ref().display())
        })?;

        Ok(())
    }

    // ========================================================================
    // Composition Helpers - Reusable setup patterns
    // ========================================================================

    /// Add a horizontal platform at the specified height
    ///
    /// Creates a platform centered at x=0 with the given width and material.
    /// Platform is 5 pixels tall.
    ///
    /// # Example
    /// ```rust,ignore
    /// ScenarioDefinition::new("Test")
    ///     .with_platform(20, 60, ValidatedMaterialId::new(1).unwrap())
    /// ```
    pub fn with_platform(mut self, y: i32, width: u32, material: ValidatedMaterialId) -> Self {
        let half_width = width as i32 / 2;
        self.setup.push(ScenarioAction::FillRect {
            min_x: -half_width,
            max_x: half_width,
            min_y: y,
            max_y: y + 5,
            material,
        });
        self
    }

    /// Teleport player to spawn position
    ///
    /// Adds a TeleportPlayer action to the setup phase.
    ///
    /// # Example
    /// ```rust,ignore
    /// ScenarioDefinition::new("Test")
    ///     .with_spawn(0.0, 100.0)
    /// ```
    pub fn with_spawn(mut self, x: f32, y: f32) -> Self {
        self.setup.push(ScenarioAction::TeleportPlayer { x, y });
        self
    }

    /// Add a rectangular chamber with walls and floor
    ///
    /// Creates a box structure with walls of the specified thickness.
    ///
    /// # Example
    /// ```rust,ignore
    /// ScenarioDefinition::new("Test")
    ///     .with_chamber(-30..30, 0..50, 4, ValidatedMaterialId::new(1).unwrap())
    /// ```
    pub fn with_chamber(
        mut self,
        x_range: std::ops::Range<i32>,
        y_range: std::ops::Range<i32>,
        wall_thickness: u32,
        material: ValidatedMaterialId,
    ) -> Self {
        let w = wall_thickness as i32;

        // Floor
        self.setup.push(ScenarioAction::FillRect {
            min_x: x_range.start,
            max_x: x_range.end,
            min_y: y_range.start,
            max_y: y_range.start + w,
            material,
        });

        // Left wall
        self.setup.push(ScenarioAction::FillRect {
            min_x: x_range.start,
            max_x: x_range.start + w,
            min_y: y_range.start,
            max_y: y_range.end,
            material,
        });

        // Right wall
        self.setup.push(ScenarioAction::FillRect {
            min_x: x_range.end - w,
            max_x: x_range.end,
            min_y: y_range.start,
            max_y: y_range.end,
            material,
        });

        // Ceiling
        self.setup.push(ScenarioAction::FillRect {
            min_x: x_range.start,
            max_x: x_range.end,
            min_y: y_range.end - w,
            max_y: y_range.end,
            material,
        });

        self
    }

    /// Create a new scenario with default empty fields
    ///
    /// Useful for building scenarios programmatically with composition helpers.
    ///
    /// # Example
    /// ```rust,ignore
    /// let scenario = ScenarioDefinition::new("Mining Test")
    ///     .with_description("Test mining mechanics")
    ///     .with_platform(20, 60, ValidatedMaterialId::new(1).unwrap())
    ///     .with_spawn(0.0, 100.0);
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            setup: Vec::new(),
            actions: Vec::new(),
            verify: Vec::new(),
            cleanup: Vec::new(),
        }
    }

    /// Set description (builder pattern)
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add an action to the main action sequence
    pub fn add_action(mut self, action: ScenarioAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Add a verification condition
    pub fn add_verification(mut self, condition: VerificationCondition) -> Self {
        self.verify.push(condition);
        self
    }

    /// Add a cleanup action
    pub fn add_cleanup(mut self, action: ScenarioAction) -> Self {
        self.cleanup.push(action);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_serialization() {
        let scenario = ScenarioDefinition {
            name: "Test Scenario".to_string(),
            description: "A test scenario".to_string(),
            setup: vec![ScenarioAction::TeleportPlayer { x: 0.0, y: 100.0 }],
            actions: vec![
                ScenarioAction::WaitFrames { frames: 60 },
                ScenarioAction::Log {
                    message: "Test message".to_string(),
                },
            ],
            verify: vec![VerificationCondition::PlayerPosition {
                x: 0.0,
                y: 100.0,
                tolerance: 5.0,
            }],
            cleanup: vec![],
        };

        // Test RON serialization
        let ron = ron::ser::to_string_pretty(&scenario, ron::ser::PrettyConfig::default()).unwrap();
        assert!(ron.contains("Test Scenario"));
        assert!(ron.contains("TeleportPlayer"));

        // Test round-trip
        let deserialized: ScenarioDefinition = ron::from_str(&ron).unwrap();
        assert_eq!(deserialized.name, scenario.name);
        assert_eq!(deserialized.actions.len(), scenario.actions.len());
    }

    #[test]
    fn test_composition_with_platform() {
        use super::super::validated_types::ValidatedMaterialId;

        let scenario = ScenarioDefinition::new("Platform Test")
            .with_description("Test platform helper")
            .with_platform(20, 60, ValidatedMaterialId::new(1).unwrap())
            .with_spawn(0.0, 100.0);

        assert_eq!(scenario.name, "Platform Test");
        assert_eq!(scenario.description, "Test platform helper");
        assert_eq!(scenario.setup.len(), 2); // FillRect + TeleportPlayer

        // Check platform was added
        match &scenario.setup[0] {
            ScenarioAction::FillRect {
                min_x,
                max_x,
                min_y,
                max_y,
                material,
            } => {
                assert_eq!(*min_x, -30); // half of 60
                assert_eq!(*max_x, 30);
                assert_eq!(*min_y, 20);
                assert_eq!(*max_y, 25); // y + 5
                assert_eq!(material.get(), 1);
            }
            _ => panic!("Expected FillRect action"),
        }

        // Check spawn was added
        match &scenario.setup[1] {
            ScenarioAction::TeleportPlayer { x, y } => {
                assert_eq!(*x, 0.0);
                assert_eq!(*y, 100.0);
            }
            _ => panic!("Expected TeleportPlayer action"),
        }
    }

    #[test]
    fn test_composition_with_chamber() {
        use super::super::validated_types::ValidatedMaterialId;

        let scenario = ScenarioDefinition::new("Chamber Test").with_chamber(
            -30..30,
            0..50,
            4,
            ValidatedMaterialId::new(1).unwrap(),
        );

        assert_eq!(scenario.setup.len(), 4); // floor + left + right + ceiling

        // All should be FillRect actions with material 1
        for action in &scenario.setup {
            match action {
                ScenarioAction::FillRect { material, .. } => {
                    assert_eq!(material.get(), 1);
                }
                _ => panic!("Expected FillRect action"),
            }
        }
    }

    #[test]
    fn test_composition_builder_pattern() {
        use super::super::verification::VerificationCondition;

        let scenario = ScenarioDefinition::new("Builder Test")
            .with_description("Test builder pattern")
            .with_spawn(0.0, 100.0)
            .add_action(ScenarioAction::WaitFrames { frames: 60 })
            .add_action(ScenarioAction::Log {
                message: "Test".to_string(),
            })
            .add_verification(VerificationCondition::PlayerPosition {
                x: 0.0,
                y: 100.0,
                tolerance: 5.0,
            })
            .add_cleanup(ScenarioAction::Log {
                message: "Cleanup".to_string(),
            });

        assert_eq!(scenario.name, "Builder Test");
        assert_eq!(scenario.setup.len(), 1);
        assert_eq!(scenario.actions.len(), 2);
        assert_eq!(scenario.verify.len(), 1);
        assert_eq!(scenario.cleanup.len(), 1);
    }

    #[test]
    fn test_composition_save_and_load() {
        use super::super::validated_types::ValidatedMaterialId;
        use tempfile::NamedTempFile;

        let scenario = ScenarioDefinition::new("Save Test")
            .with_description("Test save/load")
            .with_platform(20, 60, ValidatedMaterialId::new(1).unwrap())
            .with_spawn(0.0, 100.0)
            .add_action(ScenarioAction::WaitFrames { frames: 60 });

        // Save to temp file
        let temp_file = NamedTempFile::new().unwrap();
        scenario.to_file(temp_file.path()).unwrap();

        // Load back
        let loaded = ScenarioDefinition::from_file(temp_file.path()).unwrap();

        assert_eq!(loaded.name, "Save Test");
        assert_eq!(loaded.description, "Test save/load");
        assert_eq!(loaded.setup.len(), 2);
        assert_eq!(loaded.actions.len(), 1);
    }
}
