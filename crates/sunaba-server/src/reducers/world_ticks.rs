//! Scheduled tick reducers for world simulation, creature AI, and settlement

use glam::{IVec2, Vec2};
use spacetimedb::{ReducerContext, Table};
use std::time::Duration;

use sunaba_creature::{
    CreatureMorphology, CreaturePhysicsState, DeepNeuralController, SensorConfig, SensoryInput,
};

use crate::encoding;
use crate::helpers::{
    get_chunks_at_radius, load_or_create_chunk, sync_dirty_chunks_to_db, update_player_physics,
};
use crate::state::{NoOpStats, SERVER_WORLD};
use crate::tables::{
    CreatureData, CreatureTickTimer, Player, ServerMetrics, SettleTickTimer, WorldConfig,
    WorldTickTimer, chunk_data, creature_data, creature_tick_timer, player, server_metrics,
    settle_tick_timer, world_config, world_tick_timer,
};
use crate::world_access::SpacetimeWorldAccess;

// ============================================================================
// Timer Helpers (singleton timer pattern)
// ============================================================================

/// Delete all world tick timers (must be called before inserting new one).
/// SpacetimeDB does NOT auto-delete the timer when it fires.
fn delete_all_world_timers(ctx: &ReducerContext) {
    for timer in ctx.db.world_tick_timer().iter().collect::<Vec<_>>() {
        ctx.db.world_tick_timer().id().delete(timer.id);
    }
}

/// Delete all creature tick timers.
fn delete_all_creature_timers(ctx: &ReducerContext) {
    for timer in ctx.db.creature_tick_timer().iter().collect::<Vec<_>>() {
        ctx.db.creature_tick_timer().id().delete(timer.id);
    }
}

/// Delete all settle tick timers.
fn delete_all_settle_timers(ctx: &ReducerContext) {
    for timer in ctx.db.settle_tick_timer().iter().collect::<Vec<_>>() {
        ctx.db.settle_tick_timer().id().delete(timer.id);
    }
}

// ============================================================================
// Manual Tick Reducers (called by clients or scheduled externally)
// ============================================================================

/// World simulation tick - scheduled at 60fps (or 10fps when idle)
#[spacetimedb::reducer]
pub fn world_tick(ctx: &ReducerContext, _arg: WorldTickTimer) {
    // Delete the current timer (SpacetimeDB doesn't auto-delete on fire)
    delete_all_world_timers(ctx);

    let Some(config) = ctx.db.world_config().id().find(0) else {
        log::error!("World config not found");
        return;
    };

    if config.simulation_paused {
        return;
    }

    let new_tick_count = config.tick_count + 1;

    // Initialize or get World instance
    let mut world_guard = SERVER_WORLD.lock().unwrap();
    if world_guard.is_none() {
        // Check if chunks already exist in database
        let has_existing_chunks = ctx.db.chunk_data().iter().next().is_some();

        if has_existing_chunks {
            log::info!("Initializing server World (loading existing chunks from database)");
            let world = sunaba_core::world::World::new(false); // No terrain gen needed
            *world_guard = Some(world);
        } else {
            log::info!(
                "Initializing server World with seed {} (fresh database)",
                config.seed
            );
            let mut world = sunaba_core::world::World::new(true);
            world.set_generator(config.seed);
            *world_guard = Some(world);
        }
    }

    let Some(world) = world_guard.as_mut() else {
        log::error!("Failed to get World");
        return;
    };

    // Get online players
    let online_players: Vec<Player> = ctx.db.player().iter().filter(|p| p.online).collect();

    // Skip simulation if no players are online (reduces CPU usage when idle)
    if online_players.is_empty() {
        // Update tick count only
        ctx.db.world_config().id().update(WorldConfig {
            tick_count: new_tick_count,
            ..config
        });
        // Schedule next tick but skip simulation
        ctx.db.world_tick_timer().insert(WorldTickTimer {
            id: 0,
            scheduled_at: Duration::from_millis(100).into(), // Slower tick when idle (10fps)
        });
        return;
    }

    // Load 7x7 chunks around each player
    let mut chunks_loaded_this_tick = 0;
    for player in &online_players {
        let chunk_x = (player.x as i32).div_euclid(64);
        let chunk_y = (player.y as i32).div_euclid(64);

        let chunks_before = world.active_chunks().count();
        for dy in -3..=3 {
            for dx in -3..=3 {
                load_or_create_chunk(ctx, world, chunk_x + dx, chunk_y + dy);
            }
        }
        let chunks_after = world.active_chunks().count();
        chunks_loaded_this_tick += chunks_after - chunks_before;
    }

    // Track if new chunks were loaded (forces simulation to run)
    let new_chunks_loaded = chunks_loaded_this_tick > 0;

    // Idle detection constants
    const IDLE_THRESHOLD_TICKS: u64 = 60; // 1 second at 60fps

    // Determine if we should run full simulation
    // Run simulation if:
    // 1. World is not idle (recent activity)
    // 2. New chunks were loaded (need to settle them)
    // 3. We were previously idle and just woke up
    let should_simulate = !config.is_idle || new_chunks_loaded;

    let delta_time = 0.016;
    let mut stats = NoOpStats;
    let mut rng = ctx.rng();
    let mut dirty_chunks_synced = 0u32;

    if should_simulate {
        // Run full simulation (World::update uses dirty chunk optimization internally)
        world.update(delta_time, &mut stats, &mut rng, true);

        // Sync ONLY dirty chunks to database
        dirty_chunks_synced = sync_dirty_chunks_to_db(ctx, world, new_tick_count);
    }

    // Track world activity for idle detection
    let had_activity = dirty_chunks_synced > 0 || new_chunks_loaded;

    // Calculate new idle state
    let (last_activity_tick, is_idle) = if had_activity {
        // Reset idle timer on any activity
        if config.is_idle {
            log::info!("[TICK {}] World waking from idle mode", new_tick_count);
        }
        (new_tick_count, false)
    } else {
        // Check if we've exceeded idle threshold
        let frames_idle = new_tick_count.saturating_sub(config.last_activity_tick);
        let entering_idle = frames_idle > IDLE_THRESHOLD_TICKS && !config.is_idle;
        if entering_idle {
            log::info!(
                "[TICK {}] World entering idle mode (no activity for {} frames)",
                new_tick_count,
                frames_idle
            );
        }
        (
            config.last_activity_tick,
            frames_idle > IDLE_THRESHOLD_TICKS,
        )
    };

    // Update config with new tick count and idle state
    ctx.db.world_config().id().update(WorldConfig {
        tick_count: new_tick_count,
        last_activity_tick,
        is_idle,
        ..config
    });

    // Log statistics periodically (every 60 ticks = ~1 second at 60fps)
    if new_tick_count % 60 == 0 && !online_players.is_empty() {
        log::info!(
            "[TICK {}] {} online players, {} chunks in memory{} [{}]",
            new_tick_count,
            online_players.len(),
            world.active_chunks().count(),
            if chunks_loaded_this_tick > 0 {
                format!(" (+{} new)", chunks_loaded_this_tick)
            } else {
                String::new()
            },
            if is_idle { "IDLE" } else { "ACTIVE" }
        );
    }

    // Note: std::time::Instant is not available in WASM, so we skip timing metrics
    let world_tick_time_ms = 0.0_f32;

    // Update players (only if their chunk is loaded)
    // Player physics still runs even when idle (collision checks, etc.)
    for player in online_players {
        let chunk_x = (player.x as i32).div_euclid(64);
        let chunk_y = (player.y as i32).div_euclid(64);

        // Check if player's current chunk is loaded
        let player_chunk_loaded = world.has_chunk(IVec2::new(chunk_x, chunk_y));

        if player_chunk_loaded {
            // Only run physics if player's chunk exists
            update_player_physics(ctx, player, delta_time);
        } else {
            log::debug!(
                "Skipping physics for player at ({:.0}, {:.0}) - chunk ({}, {}) not loaded yet",
                player.x,
                player.y,
                chunk_x,
                chunk_y
            );
        }
    }

    // Collect server metrics every 10th tick (6fps sampling)
    if new_tick_count % 10 == 0 {
        // Count active chunks (loaded in world)
        let active_chunks = world.active_chunks().count() as u32;

        // Count online players
        let online_players_count = ctx.db.player().iter().filter(|p| p.online).count() as u32;

        // Count alive creatures
        let creatures_alive = ctx.db.creature_data().iter().filter(|c| c.alive).count() as u32;

        // Insert metrics sample
        ctx.db.server_metrics().insert(ServerMetrics {
            id: 0,
            tick: new_tick_count,
            timestamp_ms: (new_tick_count * 16), // Approximate timestamp (16ms per tick)
            world_tick_time_ms,                  // Measured timing from world.update()
            creature_tick_time_ms: 0.0, // Filled by creature_tick reducer (updates same tick)
            active_chunks,
            dirty_chunks_synced, // Tracked from sync_dirty_chunks_to_db()
            online_players: online_players_count,
            creatures_alive,
        });
    }

    // Cleanup old metrics every 600 ticks (10 seconds at 60fps)
    if new_tick_count % 600 == 0 {
        cleanup_old_metrics(ctx);
    }

    // Schedule next tick: 10fps when idle, 60fps when active
    let next_tick_ms = if is_idle { 100 } else { 16 };
    ctx.db.world_tick_timer().insert(WorldTickTimer {
        id: 0,
        scheduled_at: Duration::from_millis(next_tick_ms).into(),
    });
}

/// Creature AI tick - scheduled at 30fps
#[spacetimedb::reducer]
pub fn creature_tick(ctx: &ReducerContext, _arg: CreatureTickTimer) {
    // Delete the current timer (SpacetimeDB doesn't auto-delete on fire)
    delete_all_creature_timers(ctx);

    let delta_time = 0.033; // ~30fps

    // Skip creature tick if no players are online (reduces CPU usage when idle)
    let has_online_players = ctx.db.player().iter().any(|p| p.online);
    if !has_online_players {
        // Schedule next tick but skip simulation
        ctx.db.creature_tick_timer().insert(CreatureTickTimer {
            id: 0,
            scheduled_at: Duration::from_millis(200).into(), // Slower tick when idle
        });
        return;
    }

    // Get all living creatures
    let creatures: Vec<CreatureData> = ctx.db.creature_data().iter().filter(|c| c.alive).collect();

    for creature_row in creatures {
        // Deserialize creature state
        let Ok(genome) = encoding::decode_genome(&creature_row.genome_data) else {
            log::error!(
                "Failed to deserialize genome for creature {}",
                creature_row.id
            );
            continue;
        };
        let Ok(morphology) = encoding::decode_morphology(&creature_row.morphology_data) else {
            log::error!(
                "Failed to deserialize morphology for creature {}",
                creature_row.id
            );
            continue;
        };
        let Ok(mut physics_state) =
            encoding::decode_physics_state(&creature_row.physics_state_data)
        else {
            log::error!(
                "Failed to deserialize physics state for creature {}",
                creature_row.id
            );
            continue;
        };

        // Rebuild brain from genome (deterministic)
        let num_raycasts = 8;
        let num_materials = 5;
        let body_part_features = morphology.body_parts.len() * (9 + num_raycasts + num_materials);
        let output_dim = morphology.joints.len() + 1;

        let mut brain =
            DeepNeuralController::from_genome(&genome.controller, body_part_features, output_dim);

        // Create world access for sensing
        let world_access = SpacetimeWorldAccess::new(ctx);

        // Gather sensory input
        let position = Vec2::new(creature_row.x, creature_row.y);
        let sensor_config = SensorConfig::default();
        let sensory_input = SensoryInput::gather(&world_access, position, &sensor_config);

        // Extract body part features and run neural network
        let features = extract_creature_features(&morphology, &physics_state, &sensory_input);

        if features.len() == brain.input_dim() {
            let outputs = brain.forward(&features);

            // Apply motor commands
            let num_joints = morphology.joints.len();
            if outputs.len() > num_joints {
                let joint_commands: Vec<f32> = outputs[..num_joints].to_vec();
                physics_state.apply_all_motor_commands(&joint_commands, &morphology, delta_time);
            }
        }

        // Apply physics
        physics_state.apply_motor_rotations(&morphology, position);

        // Update hunger
        let mut new_hunger = creature_row.hunger - (genome.metabolic.hunger_rate * delta_time);
        let mut new_health = creature_row.health;

        // Starvation damage
        if new_hunger <= 0.0 {
            new_hunger = 0.0;
            new_health -= 5.0 * delta_time;
        }

        let alive = new_health > 0.0;

        // Serialize updated state
        let Ok(physics_state_data) = encoding::encode_physics_state(&physics_state) else {
            log::error!(
                "Failed to serialize physics state for creature {}",
                creature_row.id
            );
            continue;
        };

        // Update creature in database
        ctx.db.creature_data().id().update(CreatureData {
            physics_state_data,
            health: new_health,
            hunger: new_hunger,
            alive,
            ..creature_row
        });
    }

    // Note: std::time::Instant is not available in WASM, so timing metrics are not collected
    // The creature_tick_time_ms field in ServerMetrics will remain 0.0

    // Schedule next tick (33ms = 30fps)
    ctx.db.creature_tick_timer().insert(CreatureTickTimer {
        id: 0,
        scheduled_at: Duration::from_millis(33).into(),
    });
}

/// World settlement tick - scheduled at 10fps (low priority)
/// Pre-simulates chunks in expanding rings from spawn to prevent falling sand during exploration
#[spacetimedb::reducer]
pub fn settle_world_tick(ctx: &ReducerContext, _arg: SettleTickTimer) {
    // Delete the current timer (SpacetimeDB doesn't auto-delete on fire)
    delete_all_settle_timers(ctx);

    let Some(config) = ctx.db.world_config().id().find(0) else {
        return;
    };

    if config.settlement_complete {
        // Settlement is done, don't reschedule
        return;
    }

    // Skip settlement when no players are online (reduces CPU usage when idle)
    let has_online_players = ctx.db.player().iter().any(|p| p.online);
    if !has_online_players {
        // Reschedule at slower rate but don't do work
        ctx.db.settle_tick_timer().insert(SettleTickTimer {
            id: 0,
            scheduled_at: Duration::from_millis(1000).into(), // 1 second when idle
        });
        return;
    }

    let mut world_guard = SERVER_WORLD.lock().unwrap();
    let Some(world) = world_guard.as_mut() else {
        return;
    };

    // Settle chunks in expanding ring around spawn (0, 0)
    let r = config.settlement_progress;
    let chunks_to_settle = get_chunks_at_radius(0, 0, r);

    for (chunk_x, chunk_y) in chunks_to_settle {
        // Load chunk if not already loaded
        load_or_create_chunk(ctx, world, chunk_x, chunk_y);

        // Simulate chunk for 10 ticks (sufficient for sand/liquid settling)
        let mut rng = ctx.rng();
        for _ in 0..10 {
            world.update_chunk_settle(chunk_x, chunk_y, &mut rng);
        }

        // Save settled chunk to DB (upsert to avoid duplicate rows)
        if let Some(chunk) = world.get_chunk(chunk_x, chunk_y) {
            let Ok(pixel_data) = encoding::encode_chunk(chunk) else {
                continue;
            };

            crate::helpers::upsert_chunk(ctx, chunk_x, chunk_y, pixel_data, 0);
            log::debug!("Settled chunk ({}, {}) saved to DB", chunk_x, chunk_y);
        }
    }

    // Log ring completion
    log::info!("Settled ring {}", r);

    // Update progress
    let new_progress = r + 1;
    let complete = new_progress > config.settlement_radius;

    ctx.db.world_config().id().update(WorldConfig {
        settlement_progress: new_progress,
        settlement_complete: complete,
        ..config
    });

    // Log progress periodically (every 5 rings or on completion)
    if complete {
        let final_db_count = ctx.db.chunk_data().iter().count();
        log::info!(
            "World settlement complete! Settled radius {} ({} expected, {} in database)",
            config.settlement_radius,
            (config.settlement_radius * 2 + 1).pow(2),
            final_db_count
        );
    } else if new_progress % 5 == 0 {
        let progress_pct = (new_progress as f32 / config.settlement_radius as f32 * 100.0) as u32;
        log::info!(
            "Settlement progress: ring {}/{} ({}%)",
            new_progress,
            config.settlement_radius,
            progress_pct
        );
    }

    // Schedule next tick (100ms = 10fps)
    ctx.db.settle_tick_timer().insert(SettleTickTimer {
        id: 0,
        scheduled_at: Duration::from_millis(100).into(),
    });
}

// ============================================================================
// Burst Settlement (High Priority)
// ============================================================================

/// Settle all spawn chunks immediately in one blocking call.
/// This avoids scheduler starvation where the 60fps world_tick prevents
/// the 100ms settle_tick from ever firing.
///
/// Called on first player connect to ensure spawn area is ready.
pub fn settle_spawn_chunks_burst(ctx: &ReducerContext, radius: i32) {
    log::info!(
        "Starting burst settlement of {} rings ({} chunks)...",
        radius + 1,
        (radius * 2 + 1).pow(2)
    );

    // Initialize or get World instance
    let mut world_guard = SERVER_WORLD.lock().unwrap();
    let config = ctx
        .db
        .world_config()
        .id()
        .find(0)
        .expect("World config must exist");

    if world_guard.is_none() {
        // Check if chunks already exist in database
        let has_existing_chunks = ctx.db.chunk_data().iter().next().is_some();

        if has_existing_chunks {
            log::info!(
                "Initializing server World for burst settlement (loading existing chunks from database)"
            );
            let world = sunaba_core::world::World::new(false); // No terrain gen needed
            *world_guard = Some(world);
        } else {
            log::info!(
                "Initializing server World with seed {} for burst settlement (fresh database)",
                config.seed
            );
            let mut world = sunaba_core::world::World::new(true);
            world.set_generator(config.seed);
            *world_guard = Some(world);
        }
    }

    let world = world_guard.as_mut().expect("World must exist");
    let mut rng = ctx.rng();
    let mut chunks_settled = 0;

    // Settle ALL chunks in radius at once (no timers, blocking)
    for r in 0..=radius {
        let chunks_to_settle = get_chunks_at_radius(0, 0, r);

        for (chunk_x, chunk_y) in chunks_to_settle {
            // Load chunk if not already loaded
            load_or_create_chunk(ctx, world, chunk_x, chunk_y);

            // Simulate chunk for 10 ticks (sufficient for sand/liquid settling)
            for _ in 0..10 {
                world.update_chunk_settle(chunk_x, chunk_y, &mut rng);
            }

            // Save settled chunk to DB
            if let Some(chunk) = world.get_chunk(chunk_x, chunk_y)
                && let Ok(pixel_data) = encoding::encode_chunk(chunk)
            {
                crate::helpers::upsert_chunk(ctx, chunk_x, chunk_y, pixel_data, 0);
                chunks_settled += 1;
            }
        }
    }

    // Mark settlement as complete
    ctx.db.world_config().id().update(WorldConfig {
        settlement_progress: radius + 1,
        settlement_complete: true,
        ..config
    });

    // Cancel any pending settle timers (no longer needed)
    delete_all_settle_timers(ctx);

    log::info!(
        "Burst settlement complete! Settled {} chunks in {} rings",
        chunks_settled,
        radius + 1
    );
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Cleanup old server metrics (keep last 3600 samples = 10 minutes at 6fps)
fn cleanup_old_metrics(ctx: &ReducerContext) {
    const MAX_METRICS: usize = 3600;

    let mut metrics: Vec<ServerMetrics> = ctx.db.server_metrics().iter().collect();

    if metrics.len() > MAX_METRICS {
        metrics.sort_by_key(|m| m.tick);
        let to_remove = metrics.len() - MAX_METRICS;

        for metric in metrics.iter().take(to_remove) {
            ctx.db.server_metrics().id().delete(metric.id);
        }

        log::info!(
            "Cleaned up {} old server metrics (kept {})",
            to_remove,
            MAX_METRICS
        );
    }
}

/// Extract features from creature for neural network input
fn extract_creature_features(
    morphology: &CreatureMorphology,
    physics_state: &CreaturePhysicsState,
    sensory_input: &SensoryInput,
) -> Vec<f32> {
    let mut features = Vec::new();

    for (i, _part) in morphology.body_parts.iter().enumerate() {
        // Joint angle and velocity
        features.push(physics_state.get_motor_angle(i).unwrap_or(0.0));
        features.push(physics_state.get_motor_velocity(i).unwrap_or(0.0));

        // Position relative to root
        if let (Some(pos), Some(root_pos)) = (
            physics_state.part_positions.get(i),
            physics_state.part_positions.first(),
        ) {
            features.push((pos.x - root_pos.x) / 50.0);
            features.push((pos.y - root_pos.y) / 50.0);
        } else {
            features.push(0.0);
            features.push(0.0);
        }

        // Ground contact, food direction, etc.
        features.push(0.0); // ground contact
        // Food direction from food_direction field
        if let Some(dir) = sensory_input.food_direction {
            features.push(dir.x);
            features.push(dir.y);
        } else {
            features.push(0.0);
            features.push(0.0);
        }
        features.push(sensory_input.food_distance);
        features.push(sensory_input.gradients.food); // food gradient intensity

        // Raycast distances (8 rays)
        for ray in &sensory_input.raycasts {
            features.push(ray.distance);
        }
        let padding_count = 8_usize.saturating_sub(sensory_input.raycasts.len());
        features.extend(std::iter::repeat_n(1.0, padding_count));

        // Contact materials (5 slots)
        features.extend(std::iter::repeat_n(0.0, 5));
    }

    features
}
