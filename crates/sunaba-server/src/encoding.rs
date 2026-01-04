//! Encoding/decoding helpers for SpacetimeDB blob storage
//!
//! Uses bincode for efficient serialization of game state.

use bincode_next as bincode;
use sunaba_creature::{CreatureGenome, CreatureMorphology, CreaturePhysicsState};
use sunaba_simulation::{CHUNK_SIZE, Pixel};

/// Encode chunk pixels to bytes
pub fn encode_chunk_pixels(pixels: &[Pixel]) -> Result<Vec<u8>, String> {
    bincode::serde::encode_to_vec(pixels, bincode::config::standard())
        .map_err(|e| format!("Failed to encode pixels: {}", e))
}

/// Decode chunk pixels from bytes
pub fn decode_chunk_pixels(data: &[u8]) -> Result<Vec<Pixel>, String> {
    if data.is_empty() {
        // Return empty chunk (all air)
        return Ok(vec![Pixel::new(0); CHUNK_SIZE * CHUNK_SIZE]);
    }

    let (pixels, _): (Vec<Pixel>, _) =
        bincode::serde::decode_from_slice(data, bincode::config::standard())
            .map_err(|e| format!("Failed to decode pixels: {}", e))?;

    Ok(pixels)
}

/// Encode creature genome to bytes
pub fn encode_genome(genome: &CreatureGenome) -> Result<Vec<u8>, String> {
    bincode::serde::encode_to_vec(genome, bincode::config::standard())
        .map_err(|e| format!("Failed to encode genome: {}", e))
}

/// Decode creature genome from bytes
pub fn decode_genome(data: &[u8]) -> Result<CreatureGenome, String> {
    let (genome, _): (CreatureGenome, _) =
        bincode::serde::decode_from_slice(data, bincode::config::standard())
            .map_err(|e| format!("Failed to decode genome: {}", e))?;

    Ok(genome)
}

/// Encode creature morphology to bytes
pub fn encode_morphology(morphology: &CreatureMorphology) -> Result<Vec<u8>, String> {
    bincode::serde::encode_to_vec(morphology, bincode::config::standard())
        .map_err(|e| format!("Failed to encode morphology: {}", e))
}

/// Decode creature morphology from bytes
pub fn decode_morphology(data: &[u8]) -> Result<CreatureMorphology, String> {
    let (morphology, _): (CreatureMorphology, _) =
        bincode::serde::decode_from_slice(data, bincode::config::standard())
            .map_err(|e| format!("Failed to decode morphology: {}", e))?;

    Ok(morphology)
}

/// Encode creature physics state to bytes
pub fn encode_physics_state(state: &CreaturePhysicsState) -> Result<Vec<u8>, String> {
    bincode::serde::encode_to_vec(state, bincode::config::standard())
        .map_err(|e| format!("Failed to encode physics state: {}", e))
}

/// Decode creature physics state from bytes
pub fn decode_physics_state(data: &[u8]) -> Result<CreaturePhysicsState, String> {
    let (state, _): (CreaturePhysicsState, _) =
        bincode::serde::decode_from_slice(data, bincode::config::standard())
            .map_err(|e| format!("Failed to decode physics state: {}", e))?;

    Ok(state)
}

/// Encode full chunk (including temperature, light, etc.) to bytes
pub fn encode_chunk(chunk: &sunaba_core::world::Chunk) -> Result<Vec<u8>, String> {
    bincode::serde::encode_to_vec(chunk, bincode::config::standard())
        .map_err(|e| format!("Failed to encode chunk: {}", e))
}

/// Decode full chunk from bytes
pub fn decode_chunk(data: &[u8]) -> Result<sunaba_core::world::Chunk, String> {
    let (chunk, _): (sunaba_core::world::Chunk, _) =
        bincode::serde::decode_from_slice(data, bincode::config::standard())
            .map_err(|e| format!("Failed to decode chunk: {}", e))?;

    Ok(chunk)
}
