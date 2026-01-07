# UI Screenshot Checklist

This document tracks which UI panels need screenshots for the UI/UX design system development.

## Purpose
We need screenshots of all UI panels to:
1. Analyze current spacing, padding, and color usage
2. Identify inconsistencies across different panels
3. Design a unified UI/UX system based on best practices
4. Implement consistent styling across all panels

## How to Capture Screenshots

**Launch the game:**
```bash
just start    # or just load
```

**Open each panel using these keybindings:**
- **P** - Parameters/settings panel
- **I** - Inventory panel
- **C** - Crafting panel
- **L** - Logger panel
- **Tab** - Dock (includes worldgen editor, level selector)
- **H** - Controls help
- **Esc** - Close current panel/menu

**Capture with OS screenshot tool:**
- macOS: Shift+Cmd+5
- Windows: PrintScreen or Shift+Win+S
- Linux: PrintScreen or Shift+PrintScreen

**Save to:** `screenshots/ui_<panel_name>.png`

## UI Panels Checklist

### Core Panels (High Priority)
- [ ] **HUD** - Main heads-up display (always visible)
  - Save as: `screenshots/ui_hud.png`
  - Shows: Health bar, hunger bar, inventory count, selected material, equipped tool
  - Key: Always visible, no toggle needed

- [ ] **Inventory** - Full inventory grid (Press **I**)
  - Save as: `screenshots/ui_inventory.png`
  - Shows: 10x5 grid of slots, health/hunger stats
  - Current spacing: SLOT_SIZE=45.0, SPACING=5.0

- [ ] **Crafting** - Crafting interface (Press **C**)
  - Save as: `screenshots/ui_crafting.png`
  - Shows: Available recipes, requirements, craft buttons
  - Current width: 500.0

- [ ] **Parameters** - Settings/parameters panel (Press **P**)
  - Save as: `screenshots/ui_params.png`
  - Shows: Collapsible sections for physics, world, camera, rendering, debug
  - Current width: 300.0, spacing varies (4.0-8.0)

- [ ] **Logger** - Debug log panel (Press **L**)
  - Save as: `screenshots/ui_logger.png`
  - Shows: Log messages with levels (INFO, WARN, ERROR, DEBUG)

### Dock Panels (Medium Priority)
- [ ] **Dock** - Main dock container (Press **Tab**)
  - Save as: `screenshots/ui_dock.png`
  - Shows: Tabbed interface with level selector and worldgen editor

- [ ] **Level Selector** - Demo level selection (in Dock, press **Tab**)
  - Save as: `screenshots/ui_level_selector.png`
  - Shows: List of available demo levels with load buttons

- [ ] **WorldGen Editor** - World generation editor (in Dock, press **Tab**)
  - Save as: `screenshots/ui_worldgen.png`
  - Shows: World generation parameters and controls

### Overlay Panels (Low Priority)
- [ ] **Controls Help** - Help overlay (Press **H**)
  - Save as: `screenshots/ui_help.png`
  - Shows: Keyboard/mouse controls reference

- [ ] **Stats** - Performance stats (if visible)
  - Save as: `screenshots/ui_stats.png`
  - Shows: FPS, chunk count, entity count, etc.

- [ ] **Tooltips** - Hover tooltips
  - Save as: `screenshots/ui_tooltip.png`
  - Shows: Material/item information on hover
  - Note: Hover over a material in inventory to capture

- [ ] **Toasts** - Notification toasts
  - Save as: `screenshots/ui_toast.png`
  - Shows: Temporary notification messages
  - Note: May need to trigger an action that shows a toast

- [ ] **Game Over** - Game over screen (if applicable)
  - Save as: `screenshots/ui_game_over.png`
  - Shows: Game over state
  - Note: May need to die to trigger this

### Multiplayer Panels (Optional - if multiplayer feature enabled)
- [ ] **Multiplayer Panel** - Multiplayer status/controls
  - Save as: `screenshots/ui_multiplayer.png`
  - Shows: Server connection, player list, etc.
  - Note: Requires --features multiplayer

## Design Analysis Focus

When reviewing screenshots, look for:

### Spacing & Padding
- Internal spacing between UI elements
- External spacing/margins around panels
- Consistency across different panels
- Adherence to 8px grid system (or lack thereof)

### Colors
- Background colors (panel backgrounds, button backgrounds)
- Text colors (normal, highlighted, disabled)
- Status colors (health red, hunger yellow, success green, error red)
- Border/separator colors
- Color consistency across panels

### Typography
- Font sizes for headings, body text, labels
- Text color and contrast
- Line spacing and readability

### Visual Hierarchy
- How elements are grouped
- Use of separators, borders, and whitespace
- Button styles and affordances
- Focus states and hover effects

### Inconsistencies to Note
- Different spacing values used across panels
- Inconsistent color usage for similar elements
- Varying button styles or sizes
- Different heading treatments

## Next Steps

Once all screenshots are captured:
1. Analyze screenshots using the design principles researched
2. Document current state (spacing values, colors, inconsistencies)
3. Design unified UI/UX system based on best practices
4. Create UI_DESIGN.md with the design system specification
5. Implement consistent styling in code
6. Update CLAUDE.md with design system guidelines
