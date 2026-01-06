//! Template builder and predefined structure definitions

use crate::simulation::MaterialId;
use crate::world::structures::{AnchorType, StructureTemplate, StructureVariants};
use std::collections::HashMap;

/// Builder for creating structure templates programmatically
pub struct TemplateBuilder {
    name: String,
    pixels: Vec<(i8, i8, u16)>,
    anchor: AnchorType,
}

impl TemplateBuilder {
    /// Create a new template builder
    pub fn new(name: impl Into<String>, anchor: AnchorType) -> Self {
        Self {
            name: name.into(),
            pixels: Vec::new(),
            anchor,
        }
    }

    /// Add a single pixel at relative offset
    pub fn pixel(mut self, dx: i8, dy: i8, material: u16) -> Self {
        self.pixels.push((dx, dy, material));
        self
    }

    /// Add a horizontal line
    ///
    /// # Arguments
    /// * `y` - Y offset from anchor
    /// * `x_start` - Starting X offset (inclusive)
    /// * `x_end` - Ending X offset (inclusive)
    /// * `material` - Material ID to place
    pub fn h_line(mut self, y: i8, x_start: i8, x_end: i8, material: u16) -> Self {
        for x in x_start..=x_end {
            self.pixels.push((x, y, material));
        }
        self
    }

    /// Add a vertical line
    ///
    /// # Arguments
    /// * `x` - X offset from anchor
    /// * `y_start` - Starting Y offset (inclusive)
    /// * `y_end` - Ending Y offset (inclusive)
    /// * `material` - Material ID to place
    pub fn v_line(mut self, x: i8, y_start: i8, y_end: i8, material: u16) -> Self {
        for y in y_start..=y_end {
            self.pixels.push((x, y, material));
        }
        self
    }

    /// Add a filled rectangle
    ///
    /// # Arguments
    /// * `x_start` - Starting X offset (inclusive)
    /// * `y_start` - Starting Y offset (inclusive)
    /// * `x_end` - Ending X offset (inclusive)
    /// * `y_end` - Ending Y offset (inclusive)
    /// * `material` - Material ID to place
    pub fn rect(mut self, x_start: i8, y_start: i8, x_end: i8, y_end: i8, material: u16) -> Self {
        for y in y_start..=y_end {
            for x in x_start..=x_end {
                self.pixels.push((x, y, material));
            }
        }
        self
    }

    /// Build the template
    pub fn build(self) -> StructureTemplate {
        let bounds = calculate_bounds(&self.pixels);
        StructureTemplate {
            name: self.name,
            pixels: self.pixels,
            anchor: self.anchor,
            bounds,
            support_columns: Vec::new(),
        }
    }

    /// Build with support columns (for bridges)
    pub fn build_with_supports(self, supports: Vec<i8>) -> StructureTemplate {
        let mut template = self.build();
        template.support_columns = supports;
        template
    }
}

/// Calculate bounding box for a set of pixels
fn calculate_bounds(pixels: &[(i8, i8, u16)]) -> (i8, i8, i8, i8) {
    if pixels.is_empty() {
        return (0, 0, 0, 0);
    }

    let mut min_x = i8::MAX;
    let mut min_y = i8::MAX;
    let mut max_x = i8::MIN;
    let mut max_y = i8::MIN;

    for &(x, y, _) in pixels {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    (min_x, min_y, max_x, max_y)
}

/// Create all built-in structure templates
pub fn create_builtin_templates() -> HashMap<&'static str, StructureVariants> {
    let mut templates = HashMap::new();

    // Wooden bridges
    templates.insert("wooden_bridge", create_bridge_variants());

    // Trees (normal and marker)
    templates.insert("tree_normal", create_tree_normal_variants());
    templates.insert("tree_marker", create_tree_marker_variants());

    // Underground ruins
    templates.insert("ruin_wall", create_ruin_wall_variants());
    templates.insert("ruin_pillar", create_ruin_pillar_variants());

    templates
}

fn create_bridge_variants() -> StructureVariants {
    let variants = vec![
        // Simple plank bridge (16 wide)
        TemplateBuilder::new("bridge_simple_16", AnchorType::BottomCenter)
            .h_line(0, -8, 7, MaterialId::WOOD) // Deck
            .build_with_supports(vec![-8, 7]),
        // Medium bridge (24 wide)
        TemplateBuilder::new("bridge_medium_24", AnchorType::BottomCenter)
            .h_line(0, -12, 11, MaterialId::WOOD) // Deck layer 1
            .h_line(1, -12, 11, MaterialId::WOOD) // Deck layer 2 (double-thick)
            .build_with_supports(vec![-12, 11]),
        // Wide bridge with railings (32 wide)
        TemplateBuilder::new("bridge_railed_32", AnchorType::BottomCenter)
            .h_line(0, -16, 15, MaterialId::WOOD) // Deck
            .v_line(-16, 1, 3, MaterialId::WOOD) // Left rail
            .v_line(15, 1, 3, MaterialId::WOOD) // Right rail
            .build_with_supports(vec![-16, 15]),
    ];

    StructureVariants {
        name: "wooden_bridge".to_string(),
        variants,
    }
}

fn create_tree_normal_variants() -> StructureVariants {
    let variants = vec![
        // Small tree (height 8-12)
        TemplateBuilder::new("tree_small", AnchorType::BottomCenter)
            .v_line(0, 1, 8, MaterialId::WOOD) // Trunk
            .h_line(8, -2, 2, MaterialId::PLANT_MATTER) // Canopy layer 1
            .h_line(9, -1, 1, MaterialId::PLANT_MATTER) // Canopy layer 2
            .build(),
        // Medium tree (height 12-16)
        TemplateBuilder::new("tree_medium", AnchorType::BottomCenter)
            .v_line(0, 1, 12, MaterialId::WOOD) // Trunk
            .h_line(12, -3, 3, MaterialId::PLANT_MATTER) // Canopy layer 1
            .h_line(13, -2, 2, MaterialId::PLANT_MATTER) // Canopy layer 2
            .h_line(14, -1, 1, MaterialId::PLANT_MATTER) // Canopy layer 3
            .build(),
    ];

    StructureVariants {
        name: "tree_normal".to_string(),
        variants,
    }
}

fn create_tree_marker_variants() -> StructureVariants {
    let variants = vec![
        // Tall marker tree (height 20-25) - signals cave below
        TemplateBuilder::new("tree_marker_tall", AnchorType::BottomCenter)
            .v_line(0, 1, 20, MaterialId::WOOD) // Tall trunk
            .h_line(18, -4, 4, MaterialId::PLANT_MATTER) // Canopy layer 1 (wider)
            .h_line(19, -3, 3, MaterialId::PLANT_MATTER) // Canopy layer 2
            .h_line(20, -2, 2, MaterialId::PLANT_MATTER) // Canopy layer 3
            .build(),
    ];

    StructureVariants {
        name: "tree_marker".to_string(),
        variants,
    }
}

fn create_ruin_wall_variants() -> StructureVariants {
    let variants = vec![
        // Partial wall (8 wide, 6 tall, with gaps)
        TemplateBuilder::new("ruin_wall_partial", AnchorType::BottomCenter)
            .rect(-4, 0, -3, 5, MaterialId::STONE) // Left section
            .rect(2, 0, 3, 4, MaterialId::STONE) // Right section (shorter)
            .pixel(0, 0, MaterialId::STONE) // Center base
            .build(),
    ];

    StructureVariants {
        name: "ruin_wall".to_string(),
        variants,
    }
}

fn create_ruin_pillar_variants() -> StructureVariants {
    let variants = vec![
        // Broken pillar (2 wide, 8 tall, broken at top)
        TemplateBuilder::new("ruin_pillar", AnchorType::BottomCenter)
            .v_line(-1, 0, 7, MaterialId::STONE) // Left column
            .v_line(0, 0, 6, MaterialId::STONE) // Right column (slightly shorter)
            .build(),
    ];

    StructureVariants {
        name: "ruin_pillar".to_string(),
        variants,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_builder_basic() {
        let template = TemplateBuilder::new("test", AnchorType::BottomCenter)
            .pixel(0, 0, MaterialId::WOOD)
            .pixel(1, 0, MaterialId::WOOD)
            .build();

        assert_eq!(template.pixels.len(), 2);
        assert_eq!(template.anchor, AnchorType::BottomCenter);
        assert_eq!(template.name, "test");
    }

    #[test]
    fn test_bounds_calculation() {
        let template = TemplateBuilder::new("test", AnchorType::Center)
            .rect(-5, -5, 5, 5, MaterialId::STONE)
            .build();

        assert_eq!(template.bounds, (-5, -5, 5, 5));
    }

    #[test]
    fn test_h_line() {
        let template = TemplateBuilder::new("test", AnchorType::BottomCenter)
            .h_line(0, -2, 2, MaterialId::WOOD)
            .build();

        assert_eq!(template.pixels.len(), 5); // -2, -1, 0, 1, 2
        assert_eq!(template.bounds, (-2, 0, 2, 0));
    }

    #[test]
    fn test_v_line() {
        let template = TemplateBuilder::new("test", AnchorType::BottomCenter)
            .v_line(0, 1, 5, MaterialId::WOOD)
            .build();

        assert_eq!(template.pixels.len(), 5); // 1, 2, 3, 4, 5
        assert_eq!(template.bounds, (0, 1, 0, 5));
    }

    #[test]
    fn test_rect() {
        let template = TemplateBuilder::new("test", AnchorType::Center)
            .rect(0, 0, 2, 2, MaterialId::STONE)
            .build();

        assert_eq!(template.pixels.len(), 9); // 3x3 rectangle
        assert_eq!(template.bounds, (0, 0, 2, 2));
    }

    #[test]
    fn test_bridge_variants_created() {
        let templates = create_builtin_templates();
        let bridges = templates.get("wooden_bridge").unwrap();

        assert_eq!(bridges.len(), 3);
        assert!(!bridges.is_empty());

        // Check first variant
        let simple = &bridges.variants[0];
        assert_eq!(simple.name, "bridge_simple_16");
        assert_eq!(simple.support_columns, vec![-8, 7]);
    }

    #[test]
    fn test_tree_normal_variants_created() {
        let templates = create_builtin_templates();
        let trees = templates.get("tree_normal").unwrap();

        assert_eq!(trees.len(), 2);

        // Check small tree
        let small = &trees.variants[0];
        assert_eq!(small.name, "tree_small");
        assert!(!small.pixels.is_empty());
    }

    #[test]
    fn test_tree_marker_variants_created() {
        let templates = create_builtin_templates();
        let markers = templates.get("tree_marker").unwrap();

        assert_eq!(markers.len(), 1);

        let marker = &markers.variants[0];
        assert_eq!(marker.name, "tree_marker_tall");
        // Marker trees should be taller
        assert!(marker.height() > 15);
    }

    #[test]
    fn test_ruin_variants_created() {
        let templates = create_builtin_templates();

        let walls = templates.get("ruin_wall").unwrap();
        assert_eq!(walls.len(), 1);

        let pillars = templates.get("ruin_pillar").unwrap();
        assert_eq!(pillars.len(), 1);
    }

    #[test]
    fn test_all_builtin_templates_loaded() {
        let templates = create_builtin_templates();

        // Should have all 5 template categories
        assert_eq!(templates.len(), 5);
        assert!(templates.contains_key("wooden_bridge"));
        assert!(templates.contains_key("tree_normal"));
        assert!(templates.contains_key("tree_marker"));
        assert!(templates.contains_key("ruin_wall"));
        assert!(templates.contains_key("ruin_pillar"));
    }

    #[test]
    fn test_calculate_bounds_empty() {
        let bounds = calculate_bounds(&[]);
        assert_eq!(bounds, (0, 0, 0, 0));
    }

    #[test]
    fn test_calculate_bounds_single_pixel() {
        let bounds = calculate_bounds(&[(5, -3, MaterialId::WOOD)]);
        assert_eq!(bounds, (5, -3, 5, -3));
    }
}
