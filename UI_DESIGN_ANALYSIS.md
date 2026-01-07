# UI Design Analysis - Current State

This document analyzes the current UI implementation in Sunaba to identify patterns, inconsistencies, and opportunities for improvement.

## Design Principles (Research)

Based on industry best practices for game UI:

### 1. 8px Grid System
- **Rule**: All spacing, padding, margins, and dimensions should be multiples of 8px
- **Benefit**: Creates visual rhythm and consistency, easier to maintain
- **Common values**: 8, 16, 24, 32, 40, 48, 56, 64...

### 2. Internal ≤ External Spacing Rule
- **Rule**: Outer spacing should be equal to or greater than inner spacing
- **Example**: If internal padding is 8px, external margin should be ≥8px (typically 16px)
- **Benefit**: Creates clear visual grouping and hierarchy

### 3. Color Palette Consistency
- **Research**: Consistent color palettes increase user retention by 30%
- **Components**: Background, surface, primary, secondary, error, warning, success, text colors
- **Contrast**: WCAG AA minimum (4.5:1 for normal text, 3:1 for large text)

### 4. White Space Principles
- **Rule**: Use generous white space for readability
- **Line spacing**: 1.4-1.6x font size
- **Paragraph spacing**: 1.5-2x line height

## Current Implementation Analysis

### Inventory UI (`crates/sunaba/src/ui/inventory_ui.rs`)

**Spacing:**
- `SLOT_SIZE = 45.0` ❌ Not multiple of 8 (should be 48)
- `SPACING = 5.0` ❌ Not multiple of 8 (should be 8)
- Grid: 10 columns × 5 rows = 50 slots

**Colors:**
- Selected slot: `Color32::from_rgb(80, 80, 120)` - Blueish gray
- Empty slot: `Color32::from_rgb(40, 40, 40)` - Dark gray
- Border radius: `CornerRadius::same(4)` ❌ Not multiple of 8 (should be 8)

**Issues:**
- ❌ SLOT_SIZE (45) not on 8px grid → should be 48
- ❌ SPACING (5) not on 8px grid → should be 8
- ❌ Border radius (4) not on 8px grid → should be 8
- ✅ Grid layout is clear and organized

**Recommendations:**
```rust
const SLOT_SIZE: f32 = 48.0;  // 45 → 48 (multiple of 8)
const SPACING: f32 = 8.0;     // 5 → 8 (multiple of 8)
const BORDER_RADIUS: f32 = 8.0; // 4 → 8 (multiple of 8)
```

### Crafting UI (`crates/sunaba/src/ui/crafting_ui.rs`)

**Spacing:**
- Window width: `default_width(500.0)` ❌ Not multiple of 8 (should be 496 or 504)
- Section spacing: `ui.add_space(10.0)` ❌ Not multiple of 8 (should be 8 or 16)

**Colors:**
- Craftable (enough materials): `Color32::from_rgb(100, 255, 100)` - Bright green
- Insufficient materials: `Color32::from_rgb(255, 100, 100)` - Bright red
- Output color: `Color32::from_rgb(150, 200, 255)` - Light blue
- Tool color: `Color32::from_rgb(255, 215, 0)` - Gold

**Issues:**
- ❌ Window width (500) not on 8px grid → should be 496 or 504
- ❌ Spacing (10) not on 8px grid → should be 8 or 16
- ✅ Color coding is clear and intuitive (green=available, red=unavailable)
- ⚠️ Red/green may be problematic for colorblind users (add icons/symbols)

**Recommendations:**
```rust
.default_width(504.0)  // 500 → 504 (multiple of 8)
ui.add_space(16.0)     // 10 → 16 (multiple of 8, better visual breathing room)
```

### Parameters Panel (`crates/sunaba/src/ui/params_panel.rs`)

**Spacing:**
- Window width: `default_width(300.0)` ❌ Not multiple of 8 (should be 296 or 304)
- Small spacing: `ui.add_space(4.0)` ❌ Not on 8px grid (should be 8)
- Large spacing: `ui.add_space(8.0)` ✅ Correct (multiple of 8)

**Structure:**
- Collapsible sections: Player Physics, World Simulation, Camera, Rendering, Debug
- Sliders for numeric parameters
- Buttons for actions (Save Config, Reset to Defaults)

**Issues:**
- ❌ Window width (300) not on 8px grid → should be 304
- ❌ Small spacing (4) not on 8px grid → should be 8
- ✅ Large spacing (8) is correct
- ⚠️ Inconsistent spacing (sometimes 4, sometimes 8)

**Recommendations:**
```rust
.default_width(304.0)  // 300 → 304 (multiple of 8)
ui.add_space(8.0)      // Always use 8 for consistent small spacing
ui.add_space(16.0)     // Use 16 for larger section breaks
```

### HUD (`crates/sunaba/src/ui/hud.rs`)

**Spacing:**
- Bar spacing: `ui.add_space(5.0)` ❌ Not multiple of 8 (should be 8)

**Colors:**
- Health bar fill: `Color32::from_rgb(220, 50, 50)` - Red
- Health bar background: `Color32::from_rgb(100, 20, 20)` - Dark red
- Hunger bar fill: `Color32::from_rgb(200, 150, 50)` - Yellow/orange
- Hunger bar background: `Color32::from_rgb(80, 60, 20)` - Dark yellow/brown

**Dimensions:**
- Bar height: `20.0` ❌ Not multiple of 8 (should be 24)
- Bar width: `200.0` ✅ Multiple of 8
- Border radius: `CornerRadius::same(4)` ❌ Not multiple of 8 (should be 8)

**Issues:**
- ❌ Bar height (20) not on 8px grid → should be 24
- ❌ Bar spacing (5) not on 8px grid → should be 8
- ❌ Border radius (4) not on 8px grid → should be 8
- ✅ Bar width (200) is correct
- ✅ Color choices are intuitive (red=health, yellow=hunger)

**Recommendations:**
```rust
const BAR_HEIGHT: f32 = 24.0;    // 20 → 24 (multiple of 8)
const BAR_WIDTH: f32 = 200.0;    // Already correct
const BORDER_RADIUS: f32 = 8.0;  // 4 → 8 (multiple of 8)
ui.add_space(8.0)                // 5 → 8 (multiple of 8)
```

## Summary of Issues

### Spacing Violations (8px Grid)
1. ❌ Inventory: SLOT_SIZE (45→48), SPACING (5→8), border_radius (4→8)
2. ❌ Crafting: window_width (500→504), spacing (10→16)
3. ❌ Parameters: window_width (300→304), spacing (4→8)
4. ❌ HUD: bar_height (20→24), spacing (5→8), border_radius (4→8)

### Color Inconsistencies
- No documented color palette
- RGB values hardcoded throughout
- Different shades of gray used for backgrounds
- No centralized color constants

### Accessibility Concerns
- ⚠️ Crafting UI uses red/green for state → Add icons for colorblind users
- ⚠️ Need to verify text contrast ratios (WCAG AA: 4.5:1 minimum)

## Design System Requirements

### 1. Spacing Constants (8px Grid)
```rust
// Base spacing unit
pub const SPACING_UNIT: f32 = 8.0;

// Common spacing values (multiples of 8)
pub const SPACING_XS: f32 = 8.0;   // Extra small
pub const SPACING_SM: f32 = 16.0;  // Small
pub const SPACING_MD: f32 = 24.0;  // Medium
pub const SPACING_LG: f32 = 32.0;  // Large
pub const SPACING_XL: f32 = 40.0;  // Extra large

// Component dimensions (multiples of 8)
pub const SLOT_SIZE: f32 = 48.0;
pub const BAR_HEIGHT: f32 = 24.0;
pub const BUTTON_HEIGHT: f32 = 32.0;
pub const BORDER_RADIUS: f32 = 8.0;
```

### 2. Color Palette
```rust
// Background colors
pub const BG_PRIMARY: Color32 = Color32::from_rgb(32, 32, 32);     // Main background
pub const BG_SECONDARY: Color32 = Color32::from_rgb(40, 40, 40);   // Panel background
pub const BG_TERTIARY: Color32 = Color32::from_rgb(48, 48, 48);    // Elevated surface

// Border/separator colors
pub const BORDER_COLOR: Color32 = Color32::from_rgb(80, 80, 80);

// Text colors
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(180, 180, 180);
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(120, 120, 120);

// Status colors
pub const COLOR_SUCCESS: Color32 = Color32::from_rgb(100, 255, 100);  // Green
pub const COLOR_WARNING: Color32 = Color32::from_rgb(255, 200, 0);    // Yellow
pub const COLOR_ERROR: Color32 = Color32::from_rgb(255, 100, 100);    // Red
pub const COLOR_INFO: Color32 = Color32::from_rgb(150, 200, 255);     // Blue

// Specific status bars
pub const HEALTH_FILL: Color32 = Color32::from_rgb(220, 50, 50);
pub const HEALTH_BG: Color32 = Color32::from_rgb(100, 20, 20);
pub const HUNGER_FILL: Color32 = Color32::from_rgb(200, 150, 50);
pub const HUNGER_BG: Color32 = Color32::from_rgb(80, 60, 20);

// Interactive states
pub const SELECTED_COLOR: Color32 = Color32::from_rgb(80, 80, 120);   // Inventory selection
pub const HOVER_COLOR: Color32 = Color32::from_rgb(60, 60, 80);
```

### 3. Typography
```rust
pub const FONT_SIZE_HEADING: f32 = 18.0;
pub const FONT_SIZE_SUBHEADING: f32 = 16.0;
pub const FONT_SIZE_BODY: f32 = 14.0;
pub const FONT_SIZE_SMALL: f32 = 12.0;

pub const LINE_HEIGHT_MULTIPLIER: f32 = 1.5;
```

### 4. Component Dimensions
```rust
// Buttons
pub const BUTTON_HEIGHT: f32 = 32.0;
pub const BUTTON_MIN_WIDTH: f32 = 80.0;
pub const BUTTON_PADDING_H: f32 = 16.0;
pub const BUTTON_PADDING_V: f32 = 8.0;

// Panels
pub const PANEL_WIDTH_SM: f32 = 304.0;   // Small panel (was 300)
pub const PANEL_WIDTH_MD: f32 = 504.0;   // Medium panel (was 500)
pub const PANEL_WIDTH_LG: f32 = 704.0;   // Large panel

// Inventory
pub const SLOT_SIZE: f32 = 48.0;         // Inventory slot (was 45)
pub const SLOT_SPACING: f32 = 8.0;       // Between slots (was 5)
pub const SLOT_BORDER_RADIUS: f32 = 8.0; // Rounded corners (was 4)

// Status bars
pub const BAR_HEIGHT: f32 = 24.0;        // Status bar height (was 20)
pub const BAR_WIDTH: f32 = 200.0;        // Status bar width
pub const BAR_BORDER_RADIUS: f32 = 8.0;  // Rounded corners (was 4)
```

## Implementation Plan

### Phase 1: Create Design System Module
1. Create `crates/sunaba/src/ui/design_system.rs`
2. Define all spacing, color, and typography constants
3. Export from `ui/mod.rs`

### Phase 2: Update Existing Components
1. **Inventory UI**: Update SLOT_SIZE (45→48), SPACING (5→8), border_radius (4→8)
2. **Crafting UI**: Update window_width (500→504), spacing (10→16)
3. **Parameters Panel**: Update window_width (300→304), spacing (4→8)
4. **HUD**: Update bar_height (20→24), spacing (5→8), border_radius (4→8)

### Phase 3: Centralize Colors
1. Replace all hardcoded `Color32::from_rgb()` with design system constants
2. Ensure consistent color usage across all panels

### Phase 4: Add Accessibility Features
1. Add icons to crafting UI (in addition to red/green colors)
2. Verify text contrast ratios meet WCAG AA standards
3. Add hover states with sufficient visual distinction

### Phase 5: Documentation
1. Update CLAUDE.md with design system guidelines
2. Add code examples showing correct usage
3. Document how to maintain consistency for future UI work

## Next Steps

1. ✅ **Completed**: Research design principles
2. ✅ **Completed**: Analyze current UI code
3. ⏳ **In Progress**: Capture UI screenshots for visual validation
4. 🔲 **Pending**: Validate analysis with actual screenshots
5. 🔲 **Pending**: Create `design_system.rs` module
6. 🔲 **Pending**: Implement changes in existing UI components
7. 🔲 **Pending**: Test visual consistency across all panels
8. 🔲 **Pending**: Update CLAUDE.md with guidelines

## References

- Design for Ducks: "The Simplest Way to Get Better UI Spacing"
- Cieden: "Game UI Design Principles"
- Rejuvenate Digital: "Top 6 Game UI Design Best Practices"
- Justinmind: "8 Wonderful Tips and Examples of Game UI Design"
