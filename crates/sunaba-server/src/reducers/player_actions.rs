//! Player action reducers (movement, placement, mining, name setting)

use spacetimedb::ReducerContext;
use sunaba_simulation::{CHUNK_SIZE, Pixel};

use crate::encoding;
use crate::helpers::find_chunk_at;
use crate::tables::{ChunkData, Player, chunk_data, player};

// ============================================================================
// Player Action Reducers
// ============================================================================

/// Update player position directly (client-authoritative for now)
#[spacetimedb::reducer]
pub fn player_update_position(ctx: &ReducerContext, x: f32, y: f32, vel_x: f32, vel_y: f32) {
    let Some(player) = ctx.db.player().identity().find(ctx.sender) else {
        log::warn!("Player not found: {:?}", ctx.sender);
        return;
    };

    ctx.db.player().identity().update(Player {
        x,
        y,
        vel_x,
        vel_y,
        ..player
    });
}

/// Place a material at world coordinates
#[spacetimedb::reducer]
pub fn player_place_material(ctx: &ReducerContext, world_x: i32, world_y: i32, material_id: u16) {
    let Some(player) = ctx.db.player().identity().find(ctx.sender) else {
        log::warn!("Player not found for place: {:?}", ctx.sender);
        return;
    };

    // Distance check
    let dx = world_x as f32 - player.x;
    let dy = world_y as f32 - player.y;
    let distance = (dx * dx + dy * dy).sqrt();

    if distance > 50.0 {
        log::warn!("Player tried to place too far away");
        return;
    }

    // Get or create chunk
    let chunk_x = world_x.div_euclid(CHUNK_SIZE as i32);
    let chunk_y = world_y.div_euclid(CHUNK_SIZE as i32);
    let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = world_y.rem_euclid(CHUNK_SIZE as i32) as usize;

    if let Some(chunk) = find_chunk_at(ctx, chunk_x, chunk_y)
        && let Ok(mut pixels) = encoding::decode_chunk_pixels(&chunk.pixel_data)
    {
        let idx = local_y * CHUNK_SIZE + local_x;
        if idx < pixels.len() {
            pixels[idx] = Pixel::new(material_id);

            if let Ok(pixel_data) = encoding::encode_chunk_pixels(&pixels) {
                ctx.db.chunk_data().id().update(ChunkData {
                    pixel_data,
                    dirty: true,
                    ..chunk
                });
            }
        }
    }
}

/// Mine a pixel at world coordinates
#[spacetimedb::reducer]
pub fn player_mine(ctx: &ReducerContext, world_x: i32, world_y: i32) {
    let Some(player) = ctx.db.player().identity().find(ctx.sender) else {
        log::warn!("Player not found for mine: {:?}", ctx.sender);
        return;
    };

    // Distance check
    let dx = world_x as f32 - player.x;
    let dy = world_y as f32 - player.y;
    let distance = (dx * dx + dy * dy).sqrt();

    if distance > 50.0 {
        log::warn!("Player tried to mine too far away");
        return;
    }

    let chunk_x = world_x.div_euclid(CHUNK_SIZE as i32);
    let chunk_y = world_y.div_euclid(CHUNK_SIZE as i32);
    let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = world_y.rem_euclid(CHUNK_SIZE as i32) as usize;

    if let Some(chunk) = find_chunk_at(ctx, chunk_x, chunk_y)
        && let Ok(mut pixels) = encoding::decode_chunk_pixels(&chunk.pixel_data)
    {
        let idx = local_y * CHUNK_SIZE + local_x;
        if idx < pixels.len() {
            pixels[idx] = Pixel::new(0); // Air

            if let Ok(pixel_data) = encoding::encode_chunk_pixels(&pixels) {
                ctx.db.chunk_data().id().update(ChunkData {
                    pixel_data,
                    dirty: true,
                    ..chunk
                });
            }
        }
    }
}

/// Set player name
#[spacetimedb::reducer]
pub fn set_player_name(ctx: &ReducerContext, name: String) {
    let Some(player) = ctx.db.player().identity().find(ctx.sender) else {
        log::warn!("Player not found for set_name: {:?}", ctx.sender);
        return;
    };

    ctx.db.player().identity().update(Player {
        name: Some(name),
        ..player
    });
}
