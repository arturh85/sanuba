//! Structure template system for procedural feature placement
//!
//! Provides a flexible template-based system for placing structures like:
//! - Wooden bridges over gaps
//! - Trees (normal and marker variants)
//! - Underground ruins (walls, pillars)

use serde::{Deserialize, Serialize};

/// Compact pixel template for structures
///
/// Stores relative pixel positions from an anchor point, enabling
/// efficient serialization and variant selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureTemplate {
    /// Template identifier
    pub name: String,

    /// Pixels relative to anchor point (dx, dy, material_id)
    /// Stored as Vec for compact serialization
    pub pixels: Vec<(i8, i8, u16)>,

    /// Anchor point behavior
    pub anchor: AnchorType,

    /// Bounding box for quick bounds checking (min_x, min_y, max_x, max_y)
    pub bounds: (i8, i8, i8, i8),

    /// Optional: Support column positions for bridges (relative X offsets)
    pub support_columns: Vec<i8>,
}

impl StructureTemplate {
    /// Get the width of the structure
    pub fn width(&self) -> i32 {
        (self.bounds.2 - self.bounds.0 + 1) as i32
    }

    /// Get the height of the structure
    pub fn height(&self) -> i32 {
        (self.bounds.3 - self.bounds.1 + 1) as i32
    }
}

/// Anchor point type determines how structure is positioned in world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorType {
    /// Anchor at bottom-center pixel, grow upward (trees, towers)
    /// Requires solid ground below anchor point
    BottomCenter,

    /// Anchor at top-center pixel, grow downward (stalactites, chandeliers)
    /// Requires solid ceiling above anchor point
    TopCenter,

    /// Anchor at center of mass (generic placement)
    /// No strict requirements
    Center,

    /// Special: bridge with supports at horizontal ends
    /// Contains (left_support_x, right_support_x) relative offsets
    /// Requires solid ground at both support points
    BridgeEnds { left_offset: i8, right_offset: i8 },
}

/// Collection of template variants (randomly selected during placement)
///
/// Allows multiple variations of a structure type to be defined,
/// with selection driven by noise for determinism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureVariants {
    pub name: String,
    pub variants: Vec<StructureTemplate>,
}

impl StructureVariants {
    /// Select a variant using noise value at world position
    ///
    /// # Arguments
    /// * `noise` - Noise value in range [-1.0, 1.0]
    ///
    /// # Returns
    /// Reference to selected template variant
    pub fn select_variant(&self, noise: f64) -> &StructureTemplate {
        if self.variants.is_empty() {
            panic!("StructureVariants '{}' has no variants", self.name);
        }

        // Map noise [-1, 1] to index [0, len-1]
        let normalized = ((noise + 1.0) * 0.5).clamp(0.0, 1.0);
        let index = (normalized * self.variants.len() as f64) as usize;
        &self.variants[index.min(self.variants.len() - 1)]
    }

    /// Get number of variants
    pub fn len(&self) -> usize {
        self.variants.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.variants.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structure_template_dimensions() {
        let template = StructureTemplate {
            name: "test".to_string(),
            pixels: vec![],
            anchor: AnchorType::BottomCenter,
            bounds: (-5, -3, 5, 7),
            support_columns: vec![],
        };

        assert_eq!(template.width(), 11); // -5 to 5 inclusive
        assert_eq!(template.height(), 11); // -3 to 7 inclusive
    }

    #[test]
    fn test_anchor_type_equality() {
        assert_eq!(AnchorType::BottomCenter, AnchorType::BottomCenter);
        assert_ne!(AnchorType::BottomCenter, AnchorType::TopCenter);

        assert_eq!(
            AnchorType::BridgeEnds {
                left_offset: -10,
                right_offset: 10
            },
            AnchorType::BridgeEnds {
                left_offset: -10,
                right_offset: 10
            }
        );
    }

    #[test]
    fn test_structure_variants_select() {
        let variants = StructureVariants {
            name: "test".to_string(),
            variants: vec![
                StructureTemplate {
                    name: "variant_0".to_string(),
                    pixels: vec![],
                    anchor: AnchorType::Center,
                    bounds: (0, 0, 0, 0),
                    support_columns: vec![],
                },
                StructureTemplate {
                    name: "variant_1".to_string(),
                    pixels: vec![],
                    anchor: AnchorType::Center,
                    bounds: (0, 0, 0, 0),
                    support_columns: vec![],
                },
                StructureTemplate {
                    name: "variant_2".to_string(),
                    pixels: vec![],
                    anchor: AnchorType::Center,
                    bounds: (0, 0, 0, 0),
                    support_columns: vec![],
                },
            ],
        };

        // Test noise value mapping
        assert_eq!(variants.select_variant(-1.0).name, "variant_0");
        assert_eq!(variants.select_variant(0.0).name, "variant_1");
        assert_eq!(variants.select_variant(1.0).name, "variant_2");
    }

    #[test]
    fn test_structure_variants_len() {
        let variants = StructureVariants {
            name: "test".to_string(),
            variants: vec![StructureTemplate {
                name: "v1".to_string(),
                pixels: vec![],
                anchor: AnchorType::Center,
                bounds: (0, 0, 0, 0),
                support_columns: vec![],
            }],
        };

        assert_eq!(variants.len(), 1);
        assert!(!variants.is_empty());
    }

    #[test]
    #[should_panic(expected = "has no variants")]
    fn test_structure_variants_empty_panics() {
        let variants = StructureVariants {
            name: "empty".to_string(),
            variants: vec![],
        };

        variants.select_variant(0.0); // Should panic
    }

    #[test]
    fn test_serialization_roundtrip() {
        let template = StructureTemplate {
            name: "bridge".to_string(),
            pixels: vec![(0, 0, 42), (1, 0, 42), (2, 0, 42)],
            anchor: AnchorType::BridgeEnds {
                left_offset: -5,
                right_offset: 5,
            },
            bounds: (-5, 0, 5, 2),
            support_columns: vec![-5, 5],
        };

        // Serialize to RON
        let serialized = ron::to_string(&template).expect("Failed to serialize");

        // Deserialize back
        let deserialized: StructureTemplate =
            ron::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(template.name, deserialized.name);
        assert_eq!(template.pixels, deserialized.pixels);
        assert_eq!(template.anchor, deserialized.anchor);
        assert_eq!(template.bounds, deserialized.bounds);
        assert_eq!(template.support_columns, deserialized.support_columns);
    }
}
