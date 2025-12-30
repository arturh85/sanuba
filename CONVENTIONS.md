# Coding Conventions

## Rust Guidelines

### Error Handling
```rust
// Good: Use thiserror for library errors
#[derive(Debug, thiserror::Error)]
pub enum ChunkError {
    #[error("Chunk at ({x}, {y}) not loaded")]
    NotLoaded { x: i32, y: i32 },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Good: Propagate with context
fn load_chunk(x: i32, y: i32) -> Result<Chunk, ChunkError> {
    let path = chunk_path(x, y);
    let data = std::fs::read(&path)
        .map_err(|e| ChunkError::Io(e))?;
    // ...
}

// Bad: unwrap in library code
let chunk = chunks.get(&pos).unwrap();  // NO

// Acceptable: expect with context in binary/main
let chunk = chunks.get(&pos).expect("chunk should be loaded at player position");
```

### Naming
```rust
// Types: PascalCase
struct ChunkManager;
enum MaterialType;

// Functions/methods: snake_case
fn update_pixel();
fn get_neighbor();

// Constants: SCREAMING_SNAKE_CASE
const CHUNK_SIZE: usize = 64;
const MAX_TEMPERATURE: f32 = 10000.0;

// Module files: snake_case
// src/world/chunk_manager.rs
```

### Module Organization
```rust
// In mod.rs, re-export public items
pub mod chunk;
pub mod world;

pub use chunk::Chunk;
pub use world::World;

// Keep imports organized: std, external crates, crate modules
use std::collections::HashMap;

use wgpu::Device;
use rapier2d::prelude::*;

use crate::simulation::Materials;
```

### Documentation
```rust
/// A 64x64 region of the world containing pixel data.
/// 
/// Chunks are the unit of loading/saving and parallel simulation.
/// Each chunk maintains its own dirty rectangle for efficient updates.
/// 
/// # Example
/// ```
/// let mut chunk = Chunk::new(0, 0);
/// chunk.set_pixel(10, 10, MaterialId::SAND);
/// ```
pub struct Chunk {
    // ...
}
```

## Architecture Patterns

### Component-like Data
```rust
// Keep data flat and cache-friendly
pub struct Chunk {
    pub pixels: [Pixel; CHUNK_SIZE * CHUNK_SIZE],
    pub temperature: [f32; TEMP_GRID_SIZE * TEMP_GRID_SIZE],
    pub pressure: [f32; TEMP_GRID_SIZE * TEMP_GRID_SIZE],
    pub dirty_rect: Option<Rect>,
}

// Small, copyable pixel data
#[derive(Clone, Copy, Default)]
pub struct Pixel {
    pub material_id: u16,
    pub flags: u16,  // bitflags for state
}
```

### System Functions
```rust
// Systems are functions that operate on world state
// Keep them focused on one responsibility

pub fn update_cellular_automata(world: &mut World, dt: f32) {
    for chunk in world.active_chunks_mut() {
        update_chunk_ca(chunk, &world.materials);
    }
}

pub fn update_temperature(world: &mut World, dt: f32) {
    for chunk in world.active_chunks_mut() {
        diffuse_heat(chunk, dt);
        apply_state_changes(chunk, &world.materials);
    }
}
```

### Resource Management
```rust
// Use RAII and Drop for cleanup
pub struct GpuTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Drop for GpuTexture {
    fn drop(&mut self) {
        // wgpu handles cleanup automatically, but log for debugging
        log::debug!("Dropping GPU texture");
    }
}

// Pass references, don't clone large data
fn render_chunk(chunk: &Chunk, texture: &mut GpuTexture) {
    // ...
}
```

## Performance Patterns

### Hot Path (CA Update)
```rust
// Avoid allocations in the hot path
fn update_chunk_ca(chunk: &mut Chunk, materials: &Materials) {
    // Bad: allocating every frame
    let mut updates: Vec<(usize, Pixel)> = Vec::new();
    
    // Good: pre-allocated buffer or in-place updates
    // Use double-buffering if needed for correctness
}

// Use iterators and avoid bounds checks where safe
fn update_row(pixels: &mut [Pixel], y: usize) {
    // Compiler can often eliminate bounds checks with iterators
    for (x, pixel) in pixels.iter_mut().enumerate() {
        // ...
    }
}
```

### Parallelism
```rust
use rayon::prelude::*;

// Parallel chunk updates (checkerboard pattern)
fn update_chunks_parallel(chunks: &mut [Chunk], phase: usize) {
    chunks
        .par_iter_mut()
        .enumerate()
        .filter(|(i, _)| checkerboard_phase(*i) == phase)
        .for_each(|(_, chunk)| {
            update_chunk_ca(chunk);
        });
}
```

### GPU Upload
```rust
// Batch texture updates, don't upload every pixel change
fn sync_chunk_to_gpu(chunk: &Chunk, texture: &wgpu::Texture, queue: &wgpu::Queue) {
    if let Some(dirty_rect) = chunk.dirty_rect {
        // Only upload the dirty region
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture,
                origin: wgpu::Origin3d {
                    x: dirty_rect.x,
                    y: dirty_rect.y,
                    z: 0,
                },
                // ...
            },
            &chunk.get_region_pixels(dirty_rect),
            // ...
        );
    }
}
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn sand_falls_into_empty_space() {
        let mut chunk = Chunk::new(0, 0);
        chunk.set_pixel(10, 10, MaterialId::SAND);
        chunk.set_pixel(10, 9, MaterialId::AIR);
        
        update_chunk_ca(&mut chunk, &test_materials());
        
        assert_eq!(chunk.get_pixel(10, 10).material_id, MaterialId::AIR);
        assert_eq!(chunk.get_pixel(10, 9).material_id, MaterialId::SAND);
    }
    
    #[test]
    fn water_flows_horizontally() {
        // ...
    }
}

// Helper for tests
fn test_materials() -> Materials {
    Materials::from_embedded()  // minimal set for testing
}
```

### Integration Tests
```rust
// tests/simulation.rs
use sunaba::prelude::*;

#[test]
fn fire_spreads_to_adjacent_wood() {
    let mut world = World::new_test(100, 100);
    world.fill_rect(40, 40, 60, 60, MaterialId::WOOD);
    world.set_pixel(50, 50, MaterialId::FIRE);
    
    // Run simulation for a few seconds
    for _ in 0..180 {  // 3 seconds at 60fps
        world.update(1.0 / 60.0);
    }
    
    // Fire should have spread
    let fire_count = world.count_material(MaterialId::FIRE);
    assert!(fire_count > 10, "Fire should spread, found {fire_count} fire pixels");
}
```

## Git Conventions

### Commits
```
feat: add temperature diffusion system
fix: prevent water duplication at chunk boundaries
refactor: extract material registry into separate module
perf: parallelize chunk updates with rayon
docs: add architecture overview to CLAUDE.md
test: add integration tests for fire propagation
```

### Branches
```
main        - stable, always compiles
dev         - integration branch
feat/xxx    - feature branches
fix/xxx     - bug fixes
```

## File Organization

```
src/
├── lib.rs          # pub mod declarations, prelude
├── main.rs         # entry point only, minimal code
├── app.rs          # Application struct, main loop
├── world/
│   ├── mod.rs      # pub use re-exports
│   ├── chunk.rs    # Chunk struct and methods
│   └── world.rs    # World struct (chunk management)
└── simulation/
    ├── mod.rs
    ├── materials.rs
    └── cellular.rs
```

Each module should:
1. Have a clear single responsibility
2. Re-export public types in mod.rs
3. Keep internal helpers private
4. Document public API
