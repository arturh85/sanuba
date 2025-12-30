# Initial Prompt for Claude Code

Copy and paste this prompt to get started with Claude Code:

---

## Prompt

I'm building **Sunaba**, a 2D falling-sand physics sandbox survival game (like Noita meets Terraria). The project structure is set up with Rust + wgpu.

**Current state:**
- Basic project structure exists (see CLAUDE.md for full architecture)
- Chunk and pixel data structures implemented
- Material system with ~14 materials defined
- Basic CA simulation (sand falls, water flows, gas rises)
- wgpu renderer that displays the world texture

**Immediate next step:**
Get the project compiling and running. There may be some issues with the wgpu boilerplate or missing imports. Let's:

1. Run `cargo check` and fix any compilation errors
2. Run `cargo run` and verify we see a window with colored pixels (sand, water, stone)
3. Verify the simulation is running (sand should be falling, water should be flowing)

**After that's working**, the priorities are:
1. Add keyboard input so we can move around / spawn materials
2. Add cross-chunk pixel movement (currently pixels can't move between chunks)
3. Implement the temperature field and state changes (ice melts, water boils)
4. Add fire propagation

Please read CLAUDE.md and CONVENTIONS.md first to understand the architecture and coding style.

---

## Quick Reference

```bash
# Check compilation
cargo check

# Run in debug mode
cargo run

# Run in release mode (much faster)
cargo run --release

# Run tests
cargo test
```

Key files:
- `src/world/chunk.rs` - Pixel storage
- `src/world/world.rs` - Simulation loop
- `src/simulation/materials.rs` - Material definitions
- `src/render/renderer.rs` - wgpu rendering
