//! Underground biome zone system
//!
//! Provides depth-based underground zones with distinct themed content:
//! - Shallow Caves: Basic stone caves near the surface
//! - Mushroom Grotto: Bioluminescent fungal caves
//! - Crystal Caves: Crystalline formations with rare ores
//! - Lava Caverns: Volcanic zone with obsidian and lava pools
//! - Abyss: Deep zone near bedrock with dense ore veins

use crate::simulation::MaterialId;
use serde::{Deserialize, Serialize};

/// Underground zone types, ordered by increasing depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UndergroundZone {
    /// Surface to -500: Basic stone caves
    ShallowCaves,
    /// -500 to -1200: Bioluminescent mushroom caves
    MushroomGrotto,
    /// -1200 to -2000: Crystal formations
    CrystalCaves,
    /// -2000 to -2800: Volcanic caves with lava
    LavaCaverns,
    /// -2800 to bedrock: Deep ore-rich zone
    Abyss,
}

impl UndergroundZone {
    /// Get all zone types in order of increasing depth
    pub fn all() -> &'static [UndergroundZone] {
        &[
            UndergroundZone::ShallowCaves,
            UndergroundZone::MushroomGrotto,
            UndergroundZone::CrystalCaves,
            UndergroundZone::LavaCaverns,
            UndergroundZone::Abyss,
        ]
    }

    /// Get the display name for this zone
    pub fn name(&self) -> &'static str {
        match self {
            UndergroundZone::ShallowCaves => "Shallow Caves",
            UndergroundZone::MushroomGrotto => "Mushroom Grotto",
            UndergroundZone::CrystalCaves => "Crystal Caves",
            UndergroundZone::LavaCaverns => "Lava Caverns",
            UndergroundZone::Abyss => "Abyss",
        }
    }
}

/// Definition of an underground zone's characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneDefinition {
    /// Display name
    pub name: String,

    /// Zone type
    pub zone_type: UndergroundZone,

    /// Minimum Y coordinate (top of zone)
    pub min_y: i32,

    /// Maximum Y coordinate (bottom of zone, more negative)
    pub max_y: i32,

    /// Primary stone material (replaces regular stone)
    pub primary_stone: u16,

    /// Material for exposed cave walls
    pub cave_wall_material: u16,

    /// Feature material for decorations (mushrooms, crystals)
    pub feature_material: Option<u16>,

    /// Density of feature placement (0.0 - 1.0)
    pub feature_density: f32,

    /// Base ambient light level (0 = dark, 255 = bright)
    pub ambient_light: u8,

    /// Cave size multiplier (1.0 = normal)
    pub cave_size_multiplier: f32,

    /// Ore abundance multiplier (1.0 = normal)
    pub ore_multiplier: f32,
}

impl ZoneDefinition {
    /// Create default Shallow Caves zone
    pub fn shallow_caves() -> Self {
        Self {
            name: "Shallow Caves".to_string(),
            zone_type: UndergroundZone::ShallowCaves,
            min_y: 0,
            max_y: -500,
            primary_stone: MaterialId::STONE,
            cave_wall_material: MaterialId::STONE,
            feature_material: None,
            feature_density: 0.0,
            ambient_light: 0,
            cave_size_multiplier: 1.0,
            ore_multiplier: 1.0,
        }
    }

    /// Create default Mushroom Grotto zone
    pub fn mushroom_grotto() -> Self {
        Self {
            name: "Mushroom Grotto".to_string(),
            zone_type: UndergroundZone::MushroomGrotto,
            min_y: -500,
            max_y: -1200,
            primary_stone: MaterialId::MOSSY_STONE,
            cave_wall_material: MaterialId::MOSSY_STONE,
            feature_material: Some(MaterialId::GLOWING_MUSHROOM),
            feature_density: 0.15,
            ambient_light: 30, // Slight glow from mushrooms
            cave_size_multiplier: 1.2,
            ore_multiplier: 0.8, // Less ores, more mushrooms
        }
    }

    /// Create default Crystal Caves zone
    pub fn crystal_caves() -> Self {
        Self {
            name: "Crystal Caves".to_string(),
            zone_type: UndergroundZone::CrystalCaves,
            min_y: -1200,
            max_y: -2000,
            primary_stone: MaterialId::STONE,
            cave_wall_material: MaterialId::STONE,
            feature_material: Some(MaterialId::CRYSTAL),
            feature_density: 0.10,
            ambient_light: 20, // Crystal glow
            cave_size_multiplier: 1.0,
            ore_multiplier: 1.3, // More valuable ores
        }
    }

    /// Create default Lava Caverns zone
    pub fn lava_caverns() -> Self {
        Self {
            name: "Lava Caverns".to_string(),
            zone_type: UndergroundZone::LavaCaverns,
            min_y: -2000,
            max_y: -2800,
            primary_stone: MaterialId::BASALT,
            cave_wall_material: MaterialId::BASALT,
            feature_material: Some(MaterialId::OBSIDIAN),
            feature_density: 0.08,
            ambient_light: 40,         // Lava glow
            cave_size_multiplier: 1.4, // Larger caverns
            ore_multiplier: 1.5,       // Rich ores
        }
    }

    /// Create default Abyss zone
    pub fn abyss() -> Self {
        Self {
            name: "Abyss".to_string(),
            zone_type: UndergroundZone::Abyss,
            min_y: -2800,
            max_y: -4000, // Near bedrock
            primary_stone: MaterialId::BASALT,
            cave_wall_material: MaterialId::BASALT,
            feature_material: None,
            feature_density: 0.0,
            ambient_light: 0, // Total darkness
            cave_size_multiplier: 0.6,
            ore_multiplier: 2.0, // Dense ore veins
        }
    }

    /// Check if a Y coordinate is within this zone
    pub fn contains_y(&self, y: i32) -> bool {
        y <= self.min_y && y > self.max_y
    }

    /// Get the transition factor for blending at zone boundaries
    /// Returns 0.0 at min_y, 1.0 at max_y
    pub fn depth_factor(&self, y: i32) -> f32 {
        if !self.contains_y(y) {
            return 0.0;
        }
        let range = (self.min_y - self.max_y) as f32;
        if range == 0.0 {
            return 0.5;
        }
        (self.min_y - y) as f32 / range
    }
}

/// Registry of underground biome zones
#[derive(Debug, Clone)]
pub struct BiomeZoneRegistry {
    zones: Vec<ZoneDefinition>,
    enabled: bool,
    surface_influence: bool,
}

impl Default for BiomeZoneRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BiomeZoneRegistry {
    /// Create a new zone registry with default zones
    pub fn new() -> Self {
        Self {
            zones: vec![
                ZoneDefinition::shallow_caves(),
                ZoneDefinition::mushroom_grotto(),
                ZoneDefinition::crystal_caves(),
                ZoneDefinition::lava_caverns(),
                ZoneDefinition::abyss(),
            ],
            enabled: true,
            surface_influence: false,
        }
    }

    /// Check if zone system is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable the zone system
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if surface biomes should influence underground zones
    pub fn has_surface_influence(&self) -> bool {
        self.surface_influence
    }

    /// Enable or disable surface biome influence
    pub fn set_surface_influence(&mut self, enabled: bool) {
        self.surface_influence = enabled;
    }

    /// Get zone at a specific Y coordinate
    pub fn get_zone_at(&self, world_y: i32) -> Option<&ZoneDefinition> {
        if !self.enabled {
            return None;
        }
        self.zones.iter().find(|z| z.contains_y(world_y))
    }

    /// Get zone by type
    pub fn get_zone(&self, zone_type: UndergroundZone) -> Option<&ZoneDefinition> {
        self.zones.iter().find(|z| z.zone_type == zone_type)
    }

    /// Get mutable zone by type
    pub fn get_zone_mut(&mut self, zone_type: UndergroundZone) -> Option<&mut ZoneDefinition> {
        self.zones.iter_mut().find(|z| z.zone_type == zone_type)
    }

    /// Get the stone material for a given Y coordinate
    /// Falls back to regular stone if zones disabled or y above all zones
    pub fn get_stone_material(&self, world_y: i32) -> u16 {
        if let Some(zone) = self.get_zone_at(world_y) {
            zone.primary_stone
        } else {
            MaterialId::STONE
        }
    }

    /// Get the cave wall material for a given Y coordinate
    pub fn get_cave_wall_material(&self, world_y: i32) -> u16 {
        if let Some(zone) = self.get_zone_at(world_y) {
            zone.cave_wall_material
        } else {
            MaterialId::STONE
        }
    }

    /// Get feature material and density for decorative placement
    pub fn get_feature_info(&self, world_y: i32) -> Option<(u16, f32)> {
        self.get_zone_at(world_y)
            .and_then(|z| z.feature_material.map(|m| (m, z.feature_density)))
    }

    /// Get ore multiplier at a given depth
    pub fn get_ore_multiplier(&self, world_y: i32) -> f32 {
        if let Some(zone) = self.get_zone_at(world_y) {
            zone.ore_multiplier
        } else {
            1.0
        }
    }

    /// Get cave size multiplier at a given depth
    pub fn get_cave_size_multiplier(&self, world_y: i32) -> f32 {
        if let Some(zone) = self.get_zone_at(world_y) {
            zone.cave_size_multiplier
        } else {
            1.0
        }
    }

    /// Get ambient light level at a given depth
    pub fn get_ambient_light(&self, world_y: i32) -> u8 {
        if let Some(zone) = self.get_zone_at(world_y) {
            zone.ambient_light
        } else {
            0
        }
    }

    /// Get all zones
    pub fn zones(&self) -> &[ZoneDefinition] {
        &self.zones
    }

    /// Get mutable access to all zones
    pub fn zones_mut(&mut self) -> &mut Vec<ZoneDefinition> {
        &mut self.zones
    }

    /// Update zone from config entry
    pub fn update_zone(&mut self, zone_type: UndergroundZone, config: ZoneDefinition) {
        if let Some(zone) = self.zones.iter_mut().find(|z| z.zone_type == zone_type) {
            *zone = config;
        }
    }
}

/// Transition blending between adjacent zones
pub struct ZoneTransition {
    /// Height of transition region in pixels
    pub transition_height: i32,
}

impl Default for ZoneTransition {
    fn default() -> Self {
        Self {
            transition_height: 32,
        }
    }
}

impl ZoneTransition {
    /// Create with custom transition height
    pub fn new(transition_height: i32) -> Self {
        Self { transition_height }
    }

    /// Calculate blend factor between two zones at a given Y
    /// Returns (zone1_weight, zone2_weight) normalized to sum to 1.0
    pub fn calculate_blend(
        &self,
        world_y: i32,
        zone1: &ZoneDefinition,
        _zone2: &ZoneDefinition,
    ) -> (f32, f32) {
        // Find the boundary between zones
        let boundary_y = zone1.max_y; // zone1 is above zone2

        let half_transition = self.transition_height / 2;
        let transition_start = boundary_y + half_transition;
        let transition_end = boundary_y - half_transition;

        if world_y >= transition_start {
            // Fully in zone1
            (1.0, 0.0)
        } else if world_y <= transition_end {
            // Fully in zone2
            (0.0, 1.0)
        } else {
            // In transition region
            let t = (transition_start - world_y) as f32 / self.transition_height as f32;
            (1.0 - t, t)
        }
    }

    /// Get blended material at a boundary with noise for natural transition
    pub fn blend_materials(
        &self,
        world_y: i32,
        zone1: &ZoneDefinition,
        zone2: &ZoneDefinition,
        noise_value: f32, // -1.0 to 1.0
    ) -> u16 {
        let (_w1, w2) = self.calculate_blend(world_y, zone1, zone2);

        // Use noise to create natural-looking boundary
        let threshold = w2 + noise_value * 0.2;

        if threshold > 0.5 {
            zone2.primary_stone
        } else {
            zone1.primary_stone
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_underground_zone_all() {
        let zones = UndergroundZone::all();
        assert_eq!(zones.len(), 5);
        assert_eq!(zones[0], UndergroundZone::ShallowCaves);
        assert_eq!(zones[4], UndergroundZone::Abyss);
    }

    #[test]
    fn test_zone_definition_contains_y() {
        let zone = ZoneDefinition::mushroom_grotto();
        assert!(!zone.contains_y(0)); // Too shallow
        assert!(zone.contains_y(-500)); // At boundary (inclusive top)
        assert!(zone.contains_y(-800)); // In middle
        assert!(!zone.contains_y(-1200)); // At bottom (exclusive)
        assert!(!zone.contains_y(-1500)); // Too deep
    }

    #[test]
    fn test_zone_depth_factor() {
        let zone = ZoneDefinition::mushroom_grotto();
        // min_y = -500, max_y = -1200, range = 700

        // At top of zone
        let factor_top = zone.depth_factor(-500);
        assert!((factor_top - 0.0).abs() < 0.01);

        // At middle of zone (-850)
        let factor_mid = zone.depth_factor(-850);
        assert!((factor_mid - 0.5).abs() < 0.01);

        // Near bottom
        let factor_bottom = zone.depth_factor(-1199);
        assert!(factor_bottom > 0.9);
    }

    #[test]
    fn test_biome_zone_registry_new() {
        let registry = BiomeZoneRegistry::new();
        assert!(registry.is_enabled());
        assert!(!registry.has_surface_influence());
        assert_eq!(registry.zones().len(), 5);
    }

    #[test]
    fn test_get_zone_at() {
        let registry = BiomeZoneRegistry::new();

        // Shallow caves
        let zone = registry.get_zone_at(-100);
        assert!(zone.is_some());
        assert_eq!(zone.unwrap().zone_type, UndergroundZone::ShallowCaves);

        // Mushroom grotto
        let zone = registry.get_zone_at(-800);
        assert!(zone.is_some());
        assert_eq!(zone.unwrap().zone_type, UndergroundZone::MushroomGrotto);

        // Crystal caves
        let zone = registry.get_zone_at(-1500);
        assert!(zone.is_some());
        assert_eq!(zone.unwrap().zone_type, UndergroundZone::CrystalCaves);

        // Lava caverns
        let zone = registry.get_zone_at(-2500);
        assert!(zone.is_some());
        assert_eq!(zone.unwrap().zone_type, UndergroundZone::LavaCaverns);

        // Abyss
        let zone = registry.get_zone_at(-3000);
        assert!(zone.is_some());
        assert_eq!(zone.unwrap().zone_type, UndergroundZone::Abyss);

        // Above all zones (surface)
        let zone = registry.get_zone_at(100);
        assert!(zone.is_none());
    }

    #[test]
    fn test_get_stone_material() {
        let registry = BiomeZoneRegistry::new();

        // Shallow caves: regular stone
        assert_eq!(registry.get_stone_material(-100), MaterialId::STONE);

        // Mushroom grotto: mossy stone
        assert_eq!(registry.get_stone_material(-800), MaterialId::MOSSY_STONE);

        // Lava caverns: basalt
        assert_eq!(registry.get_stone_material(-2500), MaterialId::BASALT);
    }

    #[test]
    fn test_get_feature_info() {
        let registry = BiomeZoneRegistry::new();

        // Shallow caves: no features
        assert!(registry.get_feature_info(-100).is_none());

        // Mushroom grotto: glowing mushrooms
        let info = registry.get_feature_info(-800);
        assert!(info.is_some());
        let (material, density) = info.unwrap();
        assert_eq!(material, MaterialId::GLOWING_MUSHROOM);
        assert!(density > 0.0);
    }

    #[test]
    fn test_disabled_registry() {
        let mut registry = BiomeZoneRegistry::new();
        registry.set_enabled(false);

        assert!(registry.get_zone_at(-800).is_none());
        assert_eq!(registry.get_stone_material(-800), MaterialId::STONE);
    }

    #[test]
    fn test_zone_transition_blend() {
        let transition = ZoneTransition::new(32);
        let zone1 = ZoneDefinition::shallow_caves();
        let zone2 = ZoneDefinition::mushroom_grotto();

        // Well above boundary
        let (w1, w2) = transition.calculate_blend(-400, &zone1, &zone2);
        assert_eq!(w1, 1.0);
        assert_eq!(w2, 0.0);

        // Well below boundary
        let (w1, w2) = transition.calculate_blend(-600, &zone1, &zone2);
        assert_eq!(w1, 0.0);
        assert_eq!(w2, 1.0);

        // At boundary
        let (w1, w2) = transition.calculate_blend(-500, &zone1, &zone2);
        assert!((w1 - 0.5).abs() < 0.1);
        assert!((w2 - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_ore_multipliers() {
        let registry = BiomeZoneRegistry::new();

        // Shallow caves: normal
        assert_eq!(registry.get_ore_multiplier(-100), 1.0);

        // Abyss: rich ores
        assert_eq!(registry.get_ore_multiplier(-3000), 2.0);
    }
}
