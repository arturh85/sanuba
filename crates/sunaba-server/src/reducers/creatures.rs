//! Creature management reducers (spawning, despawning)

use glam::Vec2;
use spacetimedb::{ReducerContext, Table};
use sunaba_creature::{CreatureArchetype, CreatureGenome, CreaturePhysicsState, MorphologyConfig};
use sunaba_simulation::CHUNK_SIZE;

use crate::encoding;
use crate::tables::{CreatureData, creature_data, world_config};

// ============================================================================
// Creature Management Reducers
// ============================================================================

/// Spawn a creature from archetype
#[spacetimedb::reducer]
pub fn spawn_creature(ctx: &ReducerContext, archetype: String, x: f32, y: f32) {
    // Check creature limit
    let Some(config) = ctx.db.world_config().id().find(0) else {
        log::error!("World config not found");
        return;
    };

    let current_count = ctx.db.creature_data().iter().filter(|c| c.alive).count() as u32;
    if current_count >= config.max_creatures {
        log::warn!("Maximum creature limit reached");
        return;
    }

    // Parse archetype
    let archetype_enum = match archetype.to_lowercase().as_str() {
        "spider" => CreatureArchetype::Spider,
        "snake" => CreatureArchetype::Snake,
        "worm" => CreatureArchetype::Worm,
        "flyer" => CreatureArchetype::Flyer,
        _ => CreatureArchetype::Evolved,
    };

    // Create genome based on archetype
    let genome = match archetype_enum {
        CreatureArchetype::Spider => CreatureGenome::archetype_spider(),
        CreatureArchetype::Snake => CreatureGenome::archetype_snake(),
        CreatureArchetype::Worm => CreatureGenome::archetype_worm(),
        CreatureArchetype::Flyer => CreatureGenome::archetype_flyer(),
        CreatureArchetype::Evolved => CreatureGenome::archetype_spider(), // Default to spider for evolved
    };
    let morph_config = MorphologyConfig::default();
    let morphology = archetype_enum.create_morphology(&genome, &morph_config);
    let physics_state = CreaturePhysicsState::new(&morphology, Vec2::new(x, y));

    // Serialize
    let Ok(genome_data) = encoding::encode_genome(&genome) else {
        log::error!("Failed to serialize genome");
        return;
    };
    let Ok(morphology_data) = encoding::encode_morphology(&morphology) else {
        log::error!("Failed to serialize morphology");
        return;
    };
    let Ok(physics_state_data) = encoding::encode_physics_state(&physics_state) else {
        log::error!("Failed to serialize physics state");
        return;
    };

    // Insert creature
    ctx.db.creature_data().insert(CreatureData {
        id: 0,                        // auto_inc
        entity_id: config.tick_count, // Use tick as unique ID
        x,
        y,
        chunk_x: (x / CHUNK_SIZE as f32).floor() as i32,
        chunk_y: (y / CHUNK_SIZE as f32).floor() as i32,
        vel_x: 0.0,
        vel_y: 0.0,
        archetype,
        genome_data,
        morphology_data,
        physics_state_data,
        health: 100.0,
        max_health: 100.0,
        hunger: 100.0,
        max_hunger: 100.0,
        generation: genome.generation,
        food_eaten: 0,
        blocks_mined: 0,
        alive: true,
    });

    log::info!("Spawned creature at ({}, {})", x, y);
}
