# Research Progress

## Experiments

| Date | Experiment | Area | Status | Result |
|------|------------|------|--------|--------|
| 2026-01-03 | [Food Collection Research](2026-01-03-food-collection-research.md) | Creature/Neural | In Progress | Investigating food collection behavior |
| 2026-01-03 | [World Scale Investigation](2026-01-03-world-scale-investigation.md) | World/Rendering | Implemented | 640x360, 12px player, larger caves |

## Key Discoveries

### 2026-01-03: World Scale (Implemented)
- **Original:** ~240x135 visible, player 16px = 11.9% of screen height
- **Noita reference:** 480x270 visible, player ~10-12px = ~4% of screen height
- **New settings:**
  - Player height: 16px → 12px
  - Default zoom: 0.015 → 0.0055 (shows ~640x360 pixels)
  - Cave noise frequency: halved for larger caverns
  - Cave thresholds: lowered for more open spaces
  - **Background layer added** - Shows darkened rock behind caves (40% brightness)
- **Result:** Player now takes ~3.3% of screen (close to Noita's 4%), caves have visible depth

## Next Research Priorities

1. **Mining system changes** - 4x4 mining patches for Terraria feel
2. **Food collection creature behavior** - Continue neural/behavior research
3. **Background interactions** - Moss growing from background to foreground
