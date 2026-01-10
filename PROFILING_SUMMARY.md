# Performance Profiling Summary

## ✅ Implemented Features

### 1. Basic Performance Metrics (Always Enabled)
- **Output**: Compact JSON (~2-3KB per scenario)
- **Metrics**: avg_frame_time, peak_frame_time, update_count, durations
- **Command**: `just profile-scenario <file.ron>`
- **Use case**: Quick regression detection, CI integration

### 2. Detailed Flamegraph Profiling (Opt-in)
- **Output**: Chrome trace format (~300KB per 900 frames)
- **Visualization**: chrome://tracing or speedscope.app
- **Command**: `just profile-detailed <file.ron>`
- **Use case**: Deep performance investigation, finding bottlenecks

## Quick Start

```bash
# Basic profiling (lightweight, always available)
just profile-scenario scenarios/tier3_physics/stress_test_falling_sand.ron

# Detailed profiling (flamegraph, opt-in)
just profile-detailed scenarios/tier3_physics/stress_test_falling_sand.ron

# View flamegraph
# - Open chrome://tracing and load profiling_trace.json
# - Or drag to speedscope.app
```

## Example Workflow

### AI Agent: Check for Regressions

```bash
# After code changes
just profile-stress-tests

# Compare avg_frame_time_ms in JSON:
# Before: 10.2ms (98 FPS)
# After:  12.5ms (80 FPS) ← 23% regression!
```

### Developer: Investigate Bottleneck

```bash
# Run detailed profiling
just profile-detailed scenarios/tier3_physics/stress_test_falling_sand.ron

# Open profiling_trace.json in speedscope.app
# Find: world_update taking 2076ms on first frame
# Cause: Chunk activation spike
# Fix: Preload chunks in setup phase
```

## Performance Baseline (M1 MacBook Pro)

| Scenario | Frames | Avg (ms) | Peak (ms) | FPS | Trace Size |
|----------|--------|----------|-----------|-----|------------|
| Falling Sand | 900 | 10.2 | 2076 | 98 | 287KB |
| Smoke Test | 1 | 2.4 | 2.4 | 416 | 726B |

## Documentation

- **PROFILING_QUICKREF.md** - Commands and metrics (AI agent focused)
- **DETAILED_PROFILING.md** - Flamegraph guide (deep investigation)
- **scenarios/tier3_physics/README.md** - Stress test details

## Technical Details

### Basic Metrics (results.rs)
- Tracks frame times with `Instant::now()`
- Calculates avg/peak/total durations
- Serializes to JSON (~2KB)
- **Overhead**: <1% (just timing)

### Detailed Profiling (tracing-chrome)
- Records span entry/exit timestamps
- Generates Chrome trace format
- Optional feature: `detailed_profiling`
- **Overhead**: ~5-10% (minimal tracing)

### Instrumentation Points
- `simulate_frames` - Scenario executor
- `world_update` - Per-frame simulation
- (Add more with `tracing::info_span!`)

## Future Enhancements

Potential additions:
- [ ] Per-system breakdown (CA, temperature, chunks)
- [ ] Memory profiling (heap allocations)
- [ ] GPU profiling (wgpu integration)
- [ ] Frame time histogram (distribution analysis)
- [ ] Automated comparison (before/after diff)
- [ ] CI integration (fail on regression)

## See Also

- `just profile-scenario` - Basic profiling
- `just profile-detailed` - Flamegraph profiling
- `just profile-stress-tests` - All stress tests
- `scenarios/` - Test scenarios directory
