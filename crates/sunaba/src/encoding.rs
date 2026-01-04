//! Encoding/decoding helpers for multiplayer chunk synchronization
//!
//! Uses bincode for efficient serialization of game state.

use bincode_next as bincode;

/// Decode full chunk from bytes
pub fn decode_chunk(data: &[u8]) -> Result<sunaba_core::world::Chunk, String> {
    let (chunk, _): (sunaba_core::world::Chunk, _) =
        bincode::serde::decode_from_slice(data, bincode::config::standard())
            .map_err(|e| format!("Failed to decode chunk: {}", e))?;

    Ok(chunk)
}

/// Encode full chunk to bytes
pub fn encode_chunk(chunk: &sunaba_core::world::Chunk) -> Result<Vec<u8>, String> {
    bincode::serde::encode_to_vec(chunk, bincode::config::standard())
        .map_err(|e| format!("Failed to encode chunk: {}", e))
}
