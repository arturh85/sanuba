# Scenario Performance Profiling

This directory contains stress test scenarios designed to measure simulation performance and identify bottlenecks.

## Overview

Scenario profiling provides automated performance metrics for AI agents to:
- Detect performance regressions during development
- Identify computationally expensive operations
- Validate optimization improvements
- Ensure the simulation can handle complex scenarios

## Performance Metrics

Each scenario execution generates a JSON report with these metrics:

| Metric | Description | Typical Values |
|--------|-------------|----------------|
| `total_duration_ms` | Wall-clock time for entire scenario | 1000-20000ms |
| `setup_duration_ms` | Time to initialize world state | 0.1-10ms |
| `action_duration_ms` | Time for all actions + simulation | 1000-20000ms |
| `verification_duration_ms` | Time to run assertions | 0.1-5ms |
| `update_count` | Number of world.update() calls | 60-3600 |
| `avg_frame_time_ms` | Average time per simulation frame | 5-20ms |
| `avg_update_time_ms` | Average time per world.update() | 5-20ms |
| `peak_frame_time_ms` | Slowest single frame (find spikes) | 10-2000ms |

**Interpreting Results:**

- **Good performance**: avg_frame_time < 16.67ms (60 FPS)
- **Acceptable**: avg_frame_time < 33.33ms (30 FPS)  
- **Performance issue**: avg_frame_time > 33.33ms
- **Critical issue**: peak_frame_time > 1000ms (major spike)

## Usage

### Run stress tests with profiling:

```bash
# Single stress test
just profile-scenario scenarios/tier3_physics/stress_test_falling_sand.ron

# All stress tests
just profile-stress-tests
```

### Example output:

```
═══════════════════════════════════════════════════
Performance Report:
═══════════════════════════════════════════════════
Reading: scenario_results/Stress_Test:_Falling_Sand_result.json

Scenario: Stress Test: Falling Sand (Inverted Pyramid)
Status: ✅ PASSED
Frames: 900

Performance Metrics:
  Total Duration: 9264.43 ms
  Setup Time: 0.32 ms
  Action Time: 9263.54 ms
  Verification Time: 0.48 ms
  Updates: 900
  Avg Frame Time: 10.20 ms
  Avg Update Time: 10.29 ms
  Peak Frame Time: 2062.82 ms

Equivalent FPS: 98.0

Summary: Total: 9264ms | Avg frame: 10.2ms | Peak: 2062ms | Updates: 900
```

## Stress Test Scenarios

### 1. Falling Sand (Inverted Pyramid)
**File**: `stress_test_falling_sand.ron`

**Tests**: Cellular automata throughput with sustained sand physics

**Setup**:
- Large inverted pyramid of sand (160 blocks wide at top)
- ~30,000 sand pixels total
- Ground platform to catch falling sand

**Actions**:
- Remove apex to trigger collapse
- Simulate 900 frames (15 seconds) of falling/settling

**Expected Performance**:
- Avg frame: 8-15ms (60+ FPS)
- Peak frame: <2000ms (initial collapse is expensive)
- Total: 8000-15000ms

**Performance Indicators**:
- If avg_frame > 20ms: CA update loop bottleneck
- If peak_frame > 3000ms: Chunk activation overhead
- If setup_time > 10ms: Chunk allocation issue

### 2. Multi-Material Reaction Chamber
**File**: `stress_test_reactions.ron`

**Tests**: Material reactions + fluid dynamics under load

**Setup**:
- Large chamber (300x200 pixels)
- Alternating layers: sand, water, lava, organic matter
- Stone pillars for structural complexity

**Actions**:
- Destabilize structure (break pillars)
- Simulate 1500 frames (25 seconds) of interaction

**Expected Performance**:
- Avg frame: 10-20ms (50-100 FPS)
- Peak frame: <1500ms
- Total: 15000-30000ms

**Performance Indicators**:
- If avg_frame > 25ms: Reaction system overhead
- If peak_frame > 2000ms: Fluid pathfinding cost
- Temperature checks are expensive with many materials

## AI Agent Workflow

### 1. After Code Changes

Run stress tests to check for regressions:

```bash
just profile-stress-tests
```

Compare metrics to baseline:
- **Regression**: avg_frame increased by >20%
- **Improvement**: avg_frame decreased by >10%
- **Neutral**: Change <10%

### 2. Investigating Performance Issues

If a scenario reports poor performance:

1. **Check peak_frame_time**: Spikes indicate one-time costs (chunk loading, large reactions)
2. **Check avg_frame_time**: Sustained overhead indicates hot path bottleneck
3. **Review action_duration vs. update_count**: Calculate time per update
4. **Compare setup_duration**: High setup time means chunk allocation issue

### 3. Validating Optimizations

Before/after comparison:

```bash
# Before optimization
just profile-scenario scenarios/tier3_physics/stress_test_falling_sand.ron
# Note: Avg frame: 15.2ms, Peak: 2500ms

# Make optimization changes...

# After optimization  
just profile-scenario scenarios/tier3_physics/stress_test_falling_sand.ron
# Note: Avg frame: 10.1ms, Peak: 1800ms
# Improvement: 33% faster average, 28% lower peak
```

## Compact JSON Format

Results are stored in `scenario_results/*.json` with this structure:

```json
{
  "scenario_name": "Stress Test: Falling Sand",
  "passed": true,
  "frames_executed": 900,
  "performance": {
    "total_duration_ms": 9264.43,
    "avg_frame_time_ms": 10.20,
    "peak_frame_time_ms": 2062.82,
    "update_count": 900
  }
}
```

**Why this format?**
- Compact: ~2-3KB per result (doesn't overwhelm AI context)
- Machine-readable: Easy for AI to parse and compare
- Human-readable: Can quickly spot issues
- Version-controlled: Track performance over time

## Creating Custom Stress Tests

To add a new stress test:

1. Create `scenarios/tier3_physics/stress_test_<name>.ron`
2. Use large-scale setup (1000s of pixels)
3. Include sustained simulation (600+ frames)
4. Add verification to ensure correctness
5. Run once to establish baseline metrics
6. Document expected performance in this README

Example template:

```ron
(
    name: "Stress Test: <Your Test Name>",
    description: "Tests <specific aspect> under load",
    
    setup: [
        (type: "TeleportPlayer", x: -500.0, y: 500.0),
        // Create large-scale scenario...
    ],
    
    actions: [
        (type: "Log", message: "Starting stress test"),
        // Trigger expensive operations...
        (type: "WaitFrames", frames: 600),  // Sustain load
        (type: "CaptureScreenshot", filename: "stress_<name>.png"),
    ],
    
    verify: [
        // Verify correctness (not just performance)
    ],
)
```

## Performance Baselines

**Hardware**: M1 MacBook Pro (2021), 16GB RAM, macOS

| Scenario | Avg Frame (ms) | Peak Frame (ms) | FPS Equiv | Notes |
|----------|----------------|-----------------|-----------|-------|
| Falling Sand | 10.2 | 2062 | 98 | Peak during initial collapse |
| Reactions | TBD | TBD | TBD | Run to establish baseline |

Update these baselines when:
- Adding new stress tests
- Making major architectural changes
- Upgrading to new hardware for CI

## Troubleshooting

**Problem**: Scenario times out  
**Solution**: Increase WaitFrames timeout or reduce scenario scale

**Problem**: Peak frame time is extremely high (>5000ms)  
**Solution**: Likely chunk loading spike - preload chunks in setup

**Problem**: Avg frame time increases over time  
**Solution**: Memory leak or accumulating state - check frame_times array

**Problem**: JSON file is missing or incomplete  
**Solution**: Scenario crashed before completion - check logs

## Future Enhancements

Potential additions for better profiling:

- [ ] Memory usage tracking (heap allocations)
- [ ] Per-system breakdown (CA time, temperature time, etc.)
- [ ] Frame time histogram (distribution, not just avg/peak)
- [ ] Comparison mode (before/after optimization)
- [ ] CI integration (fail build if performance regresses)
- [ ] Flame graph generation for deep profiling
- [ ] Multi-threaded metrics (thread utilization)

## See Also

- `scenarios/` - All test scenarios
- `scenario_results/` - JSON performance reports
- `crates/sunaba/src/scenario/` - Scenario execution engine
- `justfile` - Commands for running scenarios
