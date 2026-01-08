//! Integration tests for creature terrain sensors
//!
//! Tests terrain-aware sensory inputs: ground slope, vertical clearance,
//! gap detection, and surface material sensing.

use glam::Vec2;
use sunaba_core::world::World;
use sunaba_creature::sensors::{
    SensorConfig, sense_gap_info, sense_ground_slope, sense_surface_material,
    sense_vertical_clearance,
};
use sunaba_simulation::MaterialId;

// ============================================================================
// Ground Slope Sensor Tests
// ============================================================================

#[test]
fn test_ground_slope_flat_surface() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Create flat ground at y=50
    for x in 0..200 {
        world.set_pixel(x, 50, MaterialId::STONE);
    }

    let config = SensorConfig::default();
    let position = Vec2::new(100.0, 60.0);
    let facing_direction = 1.0; // Facing right

    let slope = sense_ground_slope(&world, position, facing_direction, &config);

    // Flat ground should have slope near 0
    assert!(
        slope.abs() < 0.1,
        "Expected flat slope (~0.0), got {}",
        slope
    );
}

#[test]
fn test_ground_slope_uphill() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Create upward slope: ground rises from y=50 to y=60 over x=90 to x=110
    for x in 90..110 {
        let y = 50 + ((x - 90) / 2); // Gradual slope
        for dy in 0..5 {
            world.set_pixel(x, y - dy, MaterialId::STONE);
        }
    }

    let config = SensorConfig::default();
    let position = Vec2::new(95.0, 55.0); // On the slope
    let facing_direction = 1.0; // Facing uphill (right)

    let slope = sense_ground_slope(&world, position, facing_direction, &config);

    // Should detect positive slope (uphill)
    assert!(slope > 0.1, "Expected uphill slope (>0.1), got {}", slope);
}

#[test]
fn test_ground_slope_downhill() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Create downward slope: ground descends from y=60 to y=50 over x=90 to x=110
    for x in 90..110 {
        let y = 60 - ((x - 90) / 2); // Gradual descent
        for dy in 0..5 {
            world.set_pixel(x, y - dy, MaterialId::STONE);
        }
    }

    let config = SensorConfig::default();
    let position = Vec2::new(95.0, 60.0); // On the slope
    let facing_direction = 1.0; // Facing downhill (right)

    let slope = sense_ground_slope(&world, position, facing_direction, &config);

    // Should detect negative slope (downhill)
    assert!(
        slope < -0.1,
        "Expected downhill slope (<-0.1), got {}",
        slope
    );
}

// ============================================================================
// Vertical Clearance Sensor Tests
// ============================================================================

#[test]
fn test_vertical_clearance_open_sky() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Ground at y=50, no ceiling
    for x in 90..110 {
        world.set_pixel(x, 50, MaterialId::STONE);
    }

    let config = SensorConfig::default();
    let position = Vec2::new(100.0, 60.0); // Above ground

    let clearance = sense_vertical_clearance(&world, position, &config);

    // Open sky should have clearance = 1.0
    assert_eq!(
        clearance, 1.0,
        "Expected full clearance (1.0) for open sky, got {}",
        clearance
    );
}

#[test]
fn test_vertical_clearance_low_ceiling() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Ground at y=50, ceiling at y=65
    for x in 90..110 {
        world.set_pixel(x, 50, MaterialId::STONE); // Ground
        world.set_pixel(x, 65, MaterialId::STONE); // Ceiling
    }

    let config = SensorConfig::default();
    let position = Vec2::new(100.0, 60.0); // 5 pixels of clearance above

    let clearance = sense_vertical_clearance(&world, position, &config);

    // Should detect partial clearance
    // With default clearance_sense_height = 25, 5 pixels = 5/25 = 0.2
    let expected_clearance = 5.0 / config.clearance_sense_height;
    assert!(
        (clearance - expected_clearance).abs() < 0.05,
        "Expected clearance ~{}, got {}",
        expected_clearance,
        clearance
    );
}

#[test]
fn test_vertical_clearance_blocked() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Ground at y=50, ceiling directly above at y=61
    for x in 90..110 {
        world.set_pixel(x, 50, MaterialId::STONE); // Ground
        world.set_pixel(x, 61, MaterialId::STONE); // Ceiling right above
    }

    let config = SensorConfig::default();
    let position = Vec2::new(100.0, 60.0); // 1 pixel of clearance

    let clearance = sense_vertical_clearance(&world, position, &config);

    // Should have minimal clearance
    assert!(
        clearance < 0.1,
        "Expected blocked clearance (<0.1), got {}",
        clearance
    );
}

// ============================================================================
// Gap Detection Tests
// ============================================================================

#[test]
fn test_gap_detection_no_gap() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Continuous ground at y=50
    for x in 0..200 {
        world.set_pixel(x, 50, MaterialId::STONE);
    }

    let config = SensorConfig::default();
    let position = Vec2::new(100.0, 55.0);
    let facing_direction = 1.0;

    let (gap_distance, gap_width) = sense_gap_info(&world, position, facing_direction, &config);

    // No gap: distance=1.0 (max), width=0.0
    assert_eq!(gap_distance, 1.0, "Expected no gap (distance=1.0)");
    assert_eq!(gap_width, 0.0, "Expected no gap (width=0.0)");
}

#[test]
fn test_gap_detection_immediate_gap() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Ground with gap: y=50 for x < 105 and x > 115
    for x in 0..105 {
        world.set_pixel(x, 50, MaterialId::STONE);
    }
    for x in 115..200 {
        world.set_pixel(x, 50, MaterialId::STONE);
    }
    // Gap from x=105 to x=115 (width=10)

    let config = SensorConfig::default();
    let position = Vec2::new(104.0, 55.0); // Just before gap
    let facing_direction = 1.0; // Facing toward gap

    let (gap_distance, gap_width) = sense_gap_info(&world, position, facing_direction, &config);

    // Should detect immediate gap
    assert!(
        gap_distance < 0.1,
        "Expected immediate gap (distance<0.1), got {}",
        gap_distance
    );

    // Gap width = 10 pixels, normalized by max_gap_width (default 30)
    let expected_width = 10.0 / config.max_gap_width;
    assert!(
        (gap_width - expected_width).abs() < 0.1,
        "Expected gap width ~{}, got {}",
        expected_width,
        gap_width
    );
}

#[test]
fn test_gap_detection_distant_gap() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Ground with gap: y=50 for x < 120 and x > 130
    for x in 0..120 {
        world.set_pixel(x, 50, MaterialId::STONE);
    }
    for x in 130..200 {
        world.set_pixel(x, 50, MaterialId::STONE);
    }
    // Gap from x=120 to x=130 (width=10)

    let config = SensorConfig::default();
    let position = Vec2::new(100.0, 55.0); // 20 pixels before gap
    let facing_direction = 1.0;

    let (gap_distance, gap_width) = sense_gap_info(&world, position, facing_direction, &config);

    // Gap at distance ~20 pixels, normalized by gap_sense_distance (default 40)
    let expected_distance = 20.0 / config.gap_sense_distance;
    assert!(
        (gap_distance - expected_distance).abs() < 0.15,
        "Expected gap distance ~{}, got {}",
        expected_distance,
        gap_distance
    );

    assert!(gap_width > 0.0, "Expected gap width >0, got {}", gap_width);
}

// ============================================================================
// Surface Material Sensor Tests
// ============================================================================

#[test]
fn test_surface_material_stone() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Stone ground at y=50
    for x in 90..110 {
        world.set_pixel(x, 50, MaterialId::STONE);
    }

    let position = Vec2::new(100.0, 60.0);
    let body_radius = 5.0;

    let material = sense_surface_material(&world, position, body_radius);

    assert_eq!(material, MaterialId::STONE, "Expected stone underfoot");
}

#[test]
fn test_surface_material_sand() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Sand ground at y=50
    for x in 90..110 {
        world.set_pixel(x, 50, MaterialId::SAND);
    }

    let position = Vec2::new(100.0, 60.0);
    let body_radius = 5.0;

    let material = sense_surface_material(&world, position, body_radius);

    assert_eq!(material, MaterialId::SAND, "Expected sand underfoot");
}

#[test]
fn test_surface_material_air() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // No ground (all air)

    let position = Vec2::new(100.0, 60.0);
    let body_radius = 5.0;

    let material = sense_surface_material(&world, position, body_radius);

    assert_eq!(material, 0, "Expected air (material=0) when no ground");
}

// ============================================================================
// Determinism Test
// ============================================================================

#[test]
fn test_terrain_sensors_deterministic() {
    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Create test terrain
    for x in 90..120 {
        let y = 50 + (x - 90) / 5; // Slight slope
        for dy in 0..3 {
            world.set_pixel(x, y - dy, MaterialId::STONE);
        }
    }

    let config = SensorConfig::default();
    let position = Vec2::new(100.0, 60.0);
    let facing_direction = 1.0;
    let body_radius = 5.0;

    // Read sensors twice
    let slope1 = sense_ground_slope(&world, position, facing_direction, &config);
    let slope2 = sense_ground_slope(&world, position, facing_direction, &config);

    let clearance1 = sense_vertical_clearance(&world, position, &config);
    let clearance2 = sense_vertical_clearance(&world, position, &config);

    let (gap_dist1, gap_width1) = sense_gap_info(&world, position, facing_direction, &config);
    let (gap_dist2, gap_width2) = sense_gap_info(&world, position, facing_direction, &config);

    let material1 = sense_surface_material(&world, position, body_radius);
    let material2 = sense_surface_material(&world, position, body_radius);

    // All sensors should return identical values
    assert_eq!(slope1, slope2, "Slope sensor not deterministic");
    assert_eq!(clearance1, clearance2, "Clearance sensor not deterministic");
    assert_eq!(
        gap_dist1, gap_dist2,
        "Gap distance sensor not deterministic"
    );
    assert_eq!(gap_width1, gap_width2, "Gap width sensor not deterministic");
    assert_eq!(material1, material2, "Material sensor not deterministic");
}

// ============================================================================
// Integration Test: Feature Extraction with Terrain Sensors
// ============================================================================

#[test]
fn test_feature_extraction_includes_terrain_sensors() {
    use sunaba_creature::morphology::CreatureMorphology;
    use sunaba_creature::neural::extract_body_part_features_simple;
    use sunaba_creature::sensors::SensoryInput;
    use sunaba_creature::simple_physics::CreaturePhysicsState;

    let mut world = World::new(false);
    world.ensure_chunks_for_area(0, 0, 200, 200);

    // Create terrain with slope
    for x in 90..120 {
        let y = 50 + (x - 90) / 3; // Upward slope
        for dy in 0..5 {
            world.set_pixel(x, y - dy, MaterialId::STONE);
        }
    }

    // Create simple biped morphology
    let morphology = CreatureMorphology::test_biped();

    let config = SensorConfig::default();
    let position = Vec2::new(100.0, 60.0);

    let sensory_input = SensoryInput::gather(&world, position, &config);
    let physics_state = CreaturePhysicsState::new(&morphology, position);

    let features = extract_body_part_features_simple(
        &morphology,
        &physics_state,
        &sensory_input,
        &world,
        &config,
    );

    // Should have features for each body part
    assert_eq!(features.len(), morphology.body_parts.len());

    // Each feature should have terrain sensor data
    for feature in &features {
        // Terrain sensors should be within valid ranges
        assert!(
            feature.ground_slope >= -1.0 && feature.ground_slope <= 1.0,
            "Ground slope out of range: {}",
            feature.ground_slope
        );
        assert!(
            feature.vertical_clearance >= 0.0 && feature.vertical_clearance <= 1.0,
            "Vertical clearance out of range: {}",
            feature.vertical_clearance
        );
        assert!(
            feature.gap_distance >= 0.0 && feature.gap_distance <= 1.0,
            "Gap distance out of range: {}",
            feature.gap_distance
        );
        assert!(
            feature.gap_width >= 0.0 && feature.gap_width <= 1.0,
            "Gap width out of range: {}",
            feature.gap_width
        );
        assert!(
            feature.surface_material_encoded >= 0.0 && feature.surface_material_encoded <= 1.0,
            "Surface material encoding out of range: {}",
            feature.surface_material_encoded
        );
    }

    // At least one body part should detect the upward slope
    let has_positive_slope = features.iter().any(|f| f.ground_slope > 0.0);
    assert!(
        has_positive_slope,
        "Expected at least one body part to detect upward slope"
    );
}
