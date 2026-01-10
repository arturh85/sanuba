# AI Agent Quick Reference: Scenario Performance Profiling

## TL;DR

Automated performance testing for detecting simulation bottlenecks. Run stress tests, get compact JSON metrics (~2KB), analyze performance.

## Quick Commands

```bash
# Single stress test with formatted output
just profile-scenario scenarios/tier3_physics/stress_test_falling_sand.ron

# Detailed profiling with flamegraph (Chrome trace format)
just profile-detailed scenarios/tier3_physics/stress_test_falling_sand.ron
# Output: profiling_trace.json (view in chrome://tracing or speedscope.app)

# All stress tests
just profile-stress-tests

# Regular scenario test (no profiling output)
just test-scenario scenarios/test_mining.ron
```

## Output Format

**JSON Report** (saved to `scenario_results/<name>_result.json`):

```json
{
  "scenario_name": "Stress Test Falling Sand Inverted Pyramid",
  "passed": true,
  "frames_executed": 900,
  "performance": {
    "total_duration_ms": 9365.1,
    "setup_duration_ms": 0.3,
    "action_duration_ms": 9361.2,
    "verification_duration_ms": 3.5,
    "avg_frame_time_ms": 10.3,
    "avg_update_time_ms": 10.4,
    "peak_frame_time_ms": 2064.5,
    "update_count": 900
  }
}
```

**Formatted Terminal Output**:

```
Performance Metrics:
  Total Duration: 9365.1 ms
  Avg Frame Time: 10.3 ms
  Peak Frame Time: 2064.5 ms
  Equivalent FPS: 97.0
```

## Interpreting Results

### Good Performance
- **avg_frame_time < 16.67ms** → 60 FPS (excellent)
- **avg_frame_time < 33.33ms** → 30 FPS (acceptable)
- **peak_frame_time < 1000ms** → No major spikes

### Performance Issues
- **avg_frame_time > 33.33ms** → Bottleneck in hot path
- **peak_frame_time > 2000ms** → Expensive one-time operation
- **total_duration >> expected** → Overall slowdown

### Example Analysis

```
Scenario: Stress Test Falling Sand Inverted Pyramid
Status: ✅ PASSED
Avg Frame Time: 10.3 ms → 97 FPS (excellent)
Peak Frame Time: 2064.5 ms → Initial collapse expensive (expected)
Total Duration: 9365.1 ms → 15 seconds of simulation in 9 seconds (good)
```

**Conclusion**: Performance is good. Peak spike is expected during initial collapse.

## Available Stress Tests

| Scenario | Focus | Frames | Expected Avg (ms) |
|----------|-------|--------|-------------------|
| `stress_test_falling_sand.ron` | CA throughput | 900 | 8-15 |
| `stress_test_reactions.ron` | Material reactions | 1500 | 10-20 |

## Workflow: After Code Changes

1. **Run stress tests**:
   ```bash
   just profile-stress-tests
   ```

2. **Check for regressions**:
   - Compare `avg_frame_time_ms` to baseline
   - **Regression**: >20% slower
   - **Improvement**: >10% faster

3. **Investigate if slow**:
   - High `peak_frame_time`: One-time cost (chunk loading, large reaction)
   - High `avg_frame_time`: Hot path bottleneck (CA loop, temperature)
   - High `setup_duration`: Chunk allocation issue

## Workflow: Validating Optimizations

**Before**:
```bash
just profile-scenario scenarios/tier3_physics/stress_test_falling_sand.ron
# Note: Avg: 15.2ms, Peak: 2500ms
```

**After optimization**:
```bash
just profile-scenario scenarios/tier3_physics/stress_test_falling_sand.ron  
# Note: Avg: 10.1ms, Peak: 1800ms
# Improvement: 33% faster average, 28% lower peak
```

## Creating Custom Stress Tests

**Template** (`scenarios/tier3_physics/stress_test_custom.ron`):

```ron
(
    name: "Stress Test: Custom",
    description: "Tests <specific aspect>",
    
    setup: [
        (type: "TeleportPlayer", x: -500.0, y: 500.0),
        // Create large-scale scenario (1000s of pixels)
    ],
    
    actions: [
        (type: "Log", message: "Starting stress test"),
        // Trigger expensive operations
        (type: "WaitFrames", frames: 600),  // Sustain load
        (type: "CaptureScreenshot", filename: "stress_custom.png"),
    ],
    
    verify: [
        // Verify correctness (not just performance)
    ],
)
```

**Guidelines**:
- Large scale: 1000s of pixels
- Sustained load: 600+ frames
- Always verify correctness
- Document expected metrics

## Common Issues

**Problem**: Peak frame time >5000ms  
**Cause**: Chunk loading spike  
**Fix**: Preload chunks in setup phase

**Problem**: Avg frame time increases over time  
**Cause**: Memory leak or accumulating state  
**Fix**: Check for growing collections

**Problem**: Scenario fails but no JSON  
**Cause**: Crash during execution  
**Fix**: Check logs, reduce scenario scale

## Performance Baselines

**Hardware**: M1 MacBook Pro (2021), 16GB RAM

| Scenario | Avg (ms) | Peak (ms) | FPS |
|----------|----------|-----------|-----|
| Falling Sand | 10.3 | 2064 | 97 |
| Reactions | TBD | TBD | TBD |

Update baselines when making architectural changes.

## See Also

- `scenarios/tier3_physics/README.md` - Detailed documentation
- `scenarios/DETAILED_PROFILING.md` - Flamegraph profiling guide
- `crates/sunaba/src/scenario/` - Implementation
- `justfile` - All commands
