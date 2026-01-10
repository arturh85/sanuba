# Detailed Performance Profiling

For deeper performance analysis, Sunaba supports detailed flamegraph generation using Chrome's trace format.

## Quick Start

```bash
# Run scenario with detailed profiling
just profile-detailed scenarios/tier3_physics/stress_test_falling_sand.ron

# Output: profiling_trace.json (~300KB for 900 frames)
```

## Viewing Flamegraphs

### Option 1: Chrome Tracing (Built-in)

1. Open `chrome://tracing` in Chrome or Edge
2. Click "Load" button
3. Select `profiling_trace.json`
4. Navigate:
   - **WASD** - Pan/zoom
   - **?** - Help/shortcuts
   - Click span to see details

### Option 2: Speedscope (Recommended)

Visit [speedscope.app](https://www.speedscope.app/) and drag `profiling_trace.json` onto the page.

**Features:**
- **Time Order** view - See execution timeline
- **Left Heavy** view - Traditional flamegraph
- **Sandwich** view - Aggregate time per function
- Search function names
- Zoom/pan with mouse

## What to Look For

### 1. Hot Paths (Expensive Functions)

Wide spans = expensive operations. Look for:
- `world_update` - Main simulation loop
- CA system spans (cellular automata updates)
- Temperature simulation
- Chunk loading/activation

### 2. Frame Time Spikes

Tall stacks at specific timestamps indicate spikes. Common causes:
- Chunk loading (one-time cost)
- Large material reactions
- Structural integrity checks

### 3. Per-Frame Breakdown

Each `world_update` span shows:
- Timestamp (when it occurred)
- Duration (how long it took)
- Call stack (what was running)

## Example Analysis

**Scenario**: Stress Test Falling Sand (900 frames, 9.3s total)

**Findings** (from `profiling_trace.json`):
- `world_update` calls: 900 (expected)
- Average frame time: ~10.2ms (good)
- Peak frame time: ~2076ms (initial collapse)
- Bottleneck: First frame spike (chunk activation)

**Actionable**:
- Preload chunks in setup phase to avoid spike
- CA update loop is efficient (~10ms sustained)

## Overhead

**Profiling overhead**: ~5-10% slower with `--detailed-profiling`

| Mode | Avg Frame | File Size |
|------|-----------|-----------|
| Normal | 10.2ms | N/A |
| Profiling | 10.2ms | ~300KB/900 frames |

**Why so low?** Tracing only records span entry/exit, not full call stacks.

## Comparing Before/After

```bash
# Before optimization
just profile-detailed scenarios/tier3_physics/stress_test_falling_sand.ron
# Note peak: 2076ms, avg: 10.2ms

# Make changes...

# After optimization
just profile-detailed scenarios/tier3_physics/stress_test_falling_sand.ron
# Compare traces in speedscope side-by-side
```

## Adding Instrumentation

To add profiling to your own code:

```rust
// In functions you want to profile
#[cfg(feature = "detailed_profiling")]
let _span = tracing::info_span!("my_function_name", arg1, arg2).entered();

// Function body here...
```

**Rules**:
- Only instrument hot paths (called many times per frame)
- Keep span names short
- Drop spans explicitly if needed: `drop(_span)`

## Limitations

**What's NOT profiled:**
- GPU time (wgpu operations)
- I/O operations (file loading)
- External crate internals (unless they use tracing)

**What IS profiled:**
- All scenario executor operations
- World update loop (with instrumentation)
- Custom spans you add

## Advanced: Filtering Spans

Chrome tracing supports filtering by category:
- Filter: `cat:sunaba::scenario` - Only scenario spans
- Filter: `name:world_update` - Only specific functions

## Troubleshooting

**Problem**: Empty `profiling_trace.json`  
**Solution**: Ensure `--detailed-profiling` flag is used

**Problem**: File is huge (>1GB)  
**Solution**: Too many spans instrumented - remove some

**Problem**: Can't open in Chrome  
**Solution**: Try speedscope.app instead (more forgiving)

**Problem**: Missing function details  
**Solution**: Add more `tracing::info_span!` calls

## Performance Baseline

**Hardware**: M1 MacBook Pro (2021), 16GB RAM

| Scenario | Frames | Trace Size | Peak Function | Peak Duration |
|----------|--------|------------|---------------|---------------|
| Falling Sand | 900 | 287KB | world_update | 2076ms |
| Smoke Test | 1 | 726B | world_update | 2.4ms |

## See Also

- [Chrome Tracing Docs](https://www.chromium.org/developers/how-tos/trace-event-profiling-tool/)
- [Speedscope](https://github.com/jlfwong/speedscope)
- [tracing-chrome crate](https://docs.rs/tracing-chrome/)
