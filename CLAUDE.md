# 砂場 Sunaba - 2D Physics Sandbox Survival

## Project Vision

A 2D falling-sand survival game combining Noita's emergent physics simulation with Terraria's persistent sandbox survival gameplay. Every pixel is simulated with material properties, enabling emergent behaviors like fire spreading, water eroding, gases rising, and structures collapsing.

**Core Pillars:**
1. **Emergent Physics**: Materials behave according to their properties, not special-case code
2. **Persistent World**: Player changes persist across sessions (unlike Noita's roguelike resets)
3. **Survival Sandbox**: Terraria-style crafting, building, exploration, progression

## Technical Architecture

### Tech Stack
- **Language**: Rust (stable)
- **Graphics**: wgpu (WebGPU API, cross-platform)
- **Windowing**: winit
- **Physics (rigid bodies)**: rapier2d
- **Audio**: rodio (future)
- **Serialization**: serde + bincode (chunk persistence)

### World Structure

```
World
├── Chunks (64×64 pixels each)
│   ├── pixel_data: [u32; 4096]     // material_id (16-bit) + flags (16-bit)
│   ├── temperature: [f32; 256]      // 8×8 coarse grid for heat
│   ├── pressure: [f32; 256]         // 8×8 coarse grid for gas pressure
│   ├── dirty: bool                  // needs saving
│   └── active_rect: Option<Rect>    // dirty rectangle for updates
├── Active chunks: ~25 around player (3×3 to 5×5 grid)
├── Loaded chunks: ~100 (cached in memory)
└── Unloaded chunks: serialized to disk
```

### Simulation Layers (updated each frame)

1. **Cellular Automata** (per-pixel, 60fps target)
   - Update bottom-to-top for falling materials
   - Checkerboard pattern for parallelization
   - Material interactions and reactions

2. **Temperature/Pressure Fields** (8×8 cells per chunk, 30fps)
   - Heat diffusion between cells
   - State changes (melt, freeze, boil, condense)
   - Gas pressure equalization

3. **Structural Integrity** (event-driven, not per-frame)
   - Triggered when solid pixels removed
   - Flood-fill to find disconnected regions
   - Convert to falling rigid bodies or particles

4. **Rigid Body Physics** (rapier2d, 60fps)
   - Player, creatures, items, falling debris
   - Collision with pixel world boundary

### Material System

Materials defined in data (RON or JSON), not code:

```rust
pub struct MaterialDef {
    pub id: u16,
    pub name: String,
    pub material_type: MaterialType,  // Solid, Powder, Liquid, Gas
    pub density: f32,                 // affects falling, sinking, floating
    pub color: [u8; 4],               // RGBA
    
    // Physical properties
    pub hardness: Option<u8>,         // mining resistance (solids)
    pub friction: Option<f32>,        // sliding (powders)
    pub viscosity: Option<f32>,       // flow speed (liquids)
    
    // Thermal properties
    pub melting_point: Option<f32>,
    pub boiling_point: Option<f32>,
    pub freezing_point: Option<f32>,
    pub ignition_temp: Option<f32>,
    pub conducts_heat: f32,           // 0.0 - 1.0
    
    // State transitions
    pub melts_to: Option<u16>,        // material_id
    pub boils_to: Option<u16>,
    pub freezes_to: Option<u16>,
    pub burns_to: Option<u16>,
    pub burn_rate: Option<f32>,
    
    // Flags
    pub flammable: bool,
    pub structural: bool,             // can support other pixels
    pub conducts_electricity: bool,
}

pub enum MaterialType {
    Solid,      // doesn't move (stone, wood, metal)
    Powder,     // falls, piles up (sand, gravel, ash)
    Liquid,     // flows, seeks level (water, oil, lava)
    Gas,        // rises, disperses (steam, smoke, toxic gas)
}
```

### Chemistry/Reactions

```rust
pub struct Reaction {
    pub input_a: u16,                 // material_id
    pub input_b: u16,                 // material_id (or MATERIAL_ANY)
    pub conditions: ReactionConditions,
    pub output_a: u16,                // what input_a becomes
    pub output_b: u16,                // what input_b becomes
    pub probability: f32,             // 0.0 - 1.0 per contact per frame
}

pub struct ReactionConditions {
    pub min_temp: Option<f32>,
    pub max_temp: Option<f32>,
    pub min_pressure: Option<f32>,
    pub requires_light: bool,
}
```

Example reactions:
- water + lava → steam + stone
- acid + metal → toxic_gas + air
- fire + wood (ignition_temp) → fire + fire (spreads)
- plant + water (light) → plant + plant (growth)

### Chunk Persistence

- Chunks saved as compressed binary (bincode + lz4)
- File structure: `world/chunks/chunk_{x}_{y}.bin`
- Save on unload, load on approach
- Background thread for IO (don't block simulation)

### Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Visible pixels | 800K-1M | ~20 chunks at 64×64 |
| CA update rate | 60 fps | Parallel chunk updates |
| Temp/pressure update | 30 fps | Coarser grid, can lag |
| Rigid bodies | 100-200 | rapier2d handles easily |
| Chunk load time | <10ms | Background thread |
| Memory per chunk | ~20KB | With temp/pressure fields |

## Project Structure

```
sunaba/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Entry point, game loop
│   ├── lib.rs                  # Library root
│   ├── app.rs                  # Application state, wgpu setup
│   ├── world/
│   │   ├── mod.rs
│   │   ├── chunk.rs            # Chunk data structure
│   │   ├── world.rs            # World manager (load/unload/save)
│   │   └── generation.rs       # Procedural terrain generation
│   ├── simulation/
│   │   ├── mod.rs
│   │   ├── cellular.rs         # CA update loop
│   │   ├── materials.rs        # Material registry
│   │   ├── reactions.rs        # Chemistry system
│   │   ├── temperature.rs      # Heat diffusion
│   │   ├── pressure.rs         # Gas pressure
│   │   └── structural.rs       # Structural integrity
│   ├── physics/
│   │   ├── mod.rs
│   │   └── rigid_body.rs       # rapier2d integration
│   ├── render/
│   │   ├── mod.rs
│   │   ├── pipeline.rs         # wgpu render pipeline
│   │   ├── texture.rs          # Pixel buffer → GPU texture
│   │   └── camera.rs           # 2D camera with zoom/pan
│   ├── entity/
│   │   ├── mod.rs
│   │   ├── player.rs           # Player entity
│   │   └── creature.rs         # AI creatures (future)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── ui_state.rs         # UI state management
│   │   ├── stats.rs            # Debug stats panel (F1)
│   │   ├── tooltip.rs          # Mouse hover tooltips
│   │   └── controls_help.rs    # Help overlay (H key)
│   └── levels/
│       ├── mod.rs
│       └── demo_levels.rs      # 16 demo scenarios
├── assets/
│   ├── materials.ron           # Material definitions
│   └── reactions.ron           # Reaction definitions
└── worlds/                     # Saved world data (gitignored)
```

## Development Phases

### Phase 1: Core Simulation ✅ COMPLETED
- [x] Project setup, wgpu boilerplate
- [x] Chunk data structure
- [x] Material registry (hard-coded, RON loading deferred)
- [x] Basic CA: sand, water, stone, air
- [x] Pixel buffer rendering
- [x] Player placeholder (rectangle, WASD movement)
- [x] Camera following player
- [x] Camera zoom controls (+/-, mouse wheel)

**Note:** Material registry is fully functional with 15 materials defined in code (air, stone, sand, water, wood, fire, smoke, steam, lava, oil, acid, ice, glass, metal, bedrock). RON file loading can be added later for modding support but is not blocking progression.

**Additional Phase 1 features implemented:**
- Temperature simulation and state changes (melting, freezing, boiling)
- Fire propagation and burning mechanics
- Chemical reaction system with configurable conditions
- Debug UI with egui integration (stats, help panel, tooltips)
- Demo level system with multiple scenarios
- Temperature overlay visualization

### Phase 2: Materials & Reactions ✅ COMPLETED*
- [x] Temperature field + diffusion
- [x] State changes (melt, freeze, boil)
- [x] Fire propagation
- [x] Gas behavior (rising, disperses - pressure field exists but not fully utilized)
- [x] Reaction system
- [x] More materials (oil, acid, lava, wood, ice, glass, metal, bedrock - 15 total)

**Note:** *Gas pressure equalization infrastructure exists but is not yet utilized. This is an optional enhancement - basic gas behavior (rising/dispersing) works via cellular automata based on density.

### Phase 3: Structural Integrity ✅ COMPLETED
- [x] Anchor detection
- [x] Disconnection check
- [x] Falling debris conversion
- [x] rapier2d integration for falling chunks

**Implementation Details:**
- Event-driven structural checking (triggered when structural pixels removed)
- Bedrock material serves as indestructible anchor
- Flood-fill algorithm finds disconnected regions (max 64px radius)
- Size-based debris conversion: <50 pixels → powder particles, ≥50 pixels → rigid bodies
- Full rapier2d physics integration with debris settling and reconstruction
- 8 dedicated demo levels (levels 9-16) for stress testing structural mechanics

### Additional Features Implemented

**UI System (egui):**
- Debug stats panel (F1 key) - FPS, chunk count, temperature stats, activity metrics
- Tooltip system - shows material name and temperature at cursor position
- Controls help panel (H key) - keyboard reference overlay
- UI state management for toggling overlays

**Demo Level System:**
- 16 demo scenarios showcasing different physics behaviors
- Level navigation with N (next) and P (previous) keys
- Levels 1-8: Physics, chemistry, thermal, and reaction demos
- Levels 9-16: Structural integrity stress tests (bridges, towers, castles, etc.)
- Each level demonstrates specific emergent behaviors

**Visualization:**
- Temperature overlay (T key) - GPU-accelerated heat map with color gradient
  - Blue (frozen) → Cyan (cold) → Green (cool) → Yellow (warm) → Orange (hot) → Red (extreme)
  - 40% opacity blend over world texture
- Debris rendering with rotation and translation

**Camera Controls:**
- Keyboard zoom: +/- or numpad +/-
- Mouse wheel zoom support
- Zoom range: 0.001 (max out) to 0.5 (max in)
- Smooth multiplicative zoom (1.1x per step)

**Input System:**
- Material selection via number keys (0-9)
- Material spawning with left mouse click
- Screen-to-world coordinate conversion
- egui input filtering (prevents world interaction when UI active)

### Phase 4: World Persistence ✅ COMPLETED
- [x] Chunk serialization (bincode + lz4 compression)
- [x] Auto-save on chunk unload + periodic (60s) + manual (F5)
- [x] Cave generation with multi-octave Perlin noise
- [x] Spawn point persistence in world metadata
- [x] Command-line --regenerate flag
- [x] Game mode separation (Persistent World vs Demo Levels)
- [x] Level selector UI (L key) with dropdown menu

### Phase 5: Survival Mechanics
- [ ] Inventory system
- [ ] Crafting
- [ ] Tools (pickaxe, etc.)
- [ ] Health, hunger (maybe)
- [ ] Day/night cycle

## Coding Conventions

### Rust Style
- Use `rustfmt` defaults
- Prefer `thiserror` for error types
- Use `log` + `env_logger` for logging
- Avoid `unwrap()` in library code, use `expect()` with context or propagate errors
- Use `#[derive(Debug, Clone)]` liberally

### ECS-lite Approach
- Not using a full ECS (bevy_ecs, specs) to keep things simple
- Entities are structs with components as fields
- Systems are functions that take `&mut World` or specific components
- Can migrate to full ECS later if needed

### Performance Considerations
- Hot path (CA update) should avoid allocations
- Use `rayon` for parallel chunk updates
- Profile before optimizing - use `tracy` or `puffin`
- GPU texture upload is often the bottleneck, batch updates

## Commands

```bash
# Run in debug mode
cargo run

# Run in release mode (much faster simulation)
cargo run --release

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Lint
cargo clippy
```

## In-Game Controls

```
# Movement
WASD           : Move player

# Camera
+/-            : Zoom in/out
Mouse Wheel    : Zoom in/out

# Material Placement
0-9            : Select material to place
Left Click     : Place selected material

# World & Levels
L              : Open level selector
F5             : Manual save (persistent world only)

# UI Toggles
H              : Toggle help panel
F1             : Toggle debug stats
T              : Toggle temperature overlay
```

## Key Algorithms

### CA Update Order (Noita-style)
```
For each frame:
  1. Checkerboard pass 1: Update chunks (0,0), (0,2), (2,0), (2,2)...
  2. Checkerboard pass 2: Update chunks (0,1), (0,3), (2,1), (2,3)...
  3. Checkerboard pass 3: Update chunks (1,0), (1,2), (3,0), (3,2)...
  4. Checkerboard pass 4: Update chunks (1,1), (1,3), (3,1), (3,3)...

Within each chunk:
  For y from bottom to top:
    For x (alternating left-right each row for symmetry):
      Update pixel at (x, y)
```

### Pixel Update Logic
```rust
fn update_pixel(chunk, x, y, materials, reactions) {
    let pixel = chunk.get(x, y);
    let material = materials.get(pixel.material_id);
    
    match material.material_type {
        Powder => update_powder(chunk, x, y, material),
        Liquid => update_liquid(chunk, x, y, material),
        Gas => update_gas(chunk, x, y, material),
        Solid => {}, // solids don't move
    }
    
    // Check reactions with neighbors
    for (nx, ny) in neighbors(x, y) {
        if let Some(reaction) = find_reaction(pixel, chunk.get(nx, ny)) {
            if random() < reaction.probability {
                apply_reaction(chunk, x, y, nx, ny, reaction);
            }
        }
    }
}
```

### Structural Integrity Check
```rust
fn check_integrity(world, removed_x, removed_y) {
    // Only check solid materials
    let region = flood_fill_solids(world, removed_x, removed_y, max_radius=64);
    
    // Is any pixel in region anchored?
    for (x, y) in &region {
        if is_anchor(world, x, y) {  // bedrock, or connected to bedrock
            return;  // stable
        }
    }
    
    // Region is floating - schedule conversion
    if region.len() < 50 {
        convert_to_particles(region);  // small debris
    } else {
        convert_to_rigid_body(region);  // falling chunk
    }
}
```

## References

- [Noita GDC Talk](https://www.youtube.com/watch?v=prXuyMCgbTc) - "Exploring the Tech and Design of Noita"
- [Recreating Noita's Sand Simulation](https://www.youtube.com/watch?v=5Ka3tbbT-9E) - C/OpenGL implementation
- [Falling Sand Simulation Blog](https://blog.macuyiko.com/post/2020/an-exploration-of-cellular-automata-and-graph-based-game-systems-part-4.html)
- [wgpu Tutorial](https://sotrh.github.io/learn-wgpu/)
- [rapier2d Docs](https://rapier.rs/docs/)

## Notes for Claude

When working on this project:

1. **Start simple**: Get pixels rendering before adding complexity
2. **Profile early**: The CA loop is the hot path, measure before optimizing
3. **Data-driven materials**: Resist hardcoding material behaviors
4. **Chunk boundaries**: Most bugs will be at chunk edges - test thoroughly
5. **Determinism**: Use seeded RNG for reproducible behavior (important for debugging)
