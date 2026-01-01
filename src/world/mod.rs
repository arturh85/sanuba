//! World management - chunks, loading, saving

pub mod biome;
mod chunk;
pub mod generation;
pub mod persistence;
#[allow(clippy::module_inception)]
mod world;

pub use biome::{BiomeDefinition, BiomeRegistry, BiomeType};
pub use chunk::{pixel_flags, Chunk, Pixel, CHUNK_SIZE};
pub use generation::WorldGenerator;
pub use persistence::{ChunkPersistence, WorldMetadata};
pub use world::World;
