//! Chemical reaction system
//!
//! Handles interactions between different materials when they come into contact.
//! Examples: water + lava → steam + stone, acid + metal → air + air (corrosion)

use crate::MaterialId;
use crate::materials::Materials;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Definition of a chemical reaction between two materials
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reaction {
    /// Human-readable name
    pub name: String,

    // Input materials
    pub input_a: u16,
    pub input_b: u16,

    // Basic conditions
    pub min_temp: Option<f32>,
    pub max_temp: Option<f32>,
    pub requires_contact: bool,

    // Advanced conditions (Phase 5)
    /// Requires light level >= threshold (0-15)
    pub requires_light: Option<u8>,
    /// Minimum pressure required (for gas reactions)
    pub min_pressure: Option<f32>,
    /// Catalyst material that must be present (not consumed)
    pub catalyst: Option<u16>,

    // Output materials (what each input becomes)
    pub output_a: u16,
    pub output_b: u16,

    /// Probability per frame when conditions are met (0.0 - 1.0)
    /// Lower values make reactions gradual rather than instant
    pub probability: f32,

    /// Heat released/absorbed (positive = exothermic, negative = endothermic)
    pub energy_released: f32,
}

/// Registry of all possible reactions with O(1) lookup via HashMap
/// Key: (material_a, material_b) where material_a <= material_b (normalized order)
/// Value: Vec of reactions possible between these materials
pub struct ReactionRegistry {
    reactions: HashMap<(u16, u16), Vec<Reaction>>,
}

impl ReactionRegistry {
    pub fn new(materials: &Materials) -> Self {
        let mut registry = Self {
            reactions: HashMap::new(),
        };
        registry.register_default_reactions(materials);
        registry
    }

    /// Register all default reactions
    fn register_default_reactions(&mut self, materials: &Materials) {
        // ===== ORIGINAL REACTIONS (updated with new fields) =====

        // Water + Lava → Steam + Stone
        self.register(Reaction {
            name: "water_lava_steam".to_string(),
            input_a: MaterialId::WATER,
            input_b: MaterialId::LAVA,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::STEAM,
            output_b: MaterialId::STONE,
            probability: 0.3,
            energy_released: -100.0, // Endothermic (absorbs heat from lava)
        });

        // Acid + Metal → Air + Air (corrosion)
        self.register(Reaction {
            name: "acid_metal_corrode".to_string(),
            input_a: MaterialId::ACID,
            input_b: MaterialId::METAL,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::AIR,
            output_b: MaterialId::AIR,
            probability: 0.05,
            energy_released: 0.0,
        });

        // Acid + Stone → Acid + Air
        self.register(Reaction {
            name: "acid_stone_corrode".to_string(),
            input_a: MaterialId::ACID,
            input_b: MaterialId::STONE,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::ACID,
            output_b: MaterialId::AIR,
            probability: 0.01,
            energy_released: 0.0,
        });

        // Acid + Wood → Acid + Air
        self.register(Reaction {
            name: "acid_wood_corrode".to_string(),
            input_a: MaterialId::ACID,
            input_b: MaterialId::WOOD,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::ACID,
            output_b: MaterialId::AIR,
            probability: 0.03,
            energy_released: 0.0,
        });

        // Ice + Lava → Water + Stone
        self.register(Reaction {
            name: "ice_lava_cool".to_string(),
            input_a: MaterialId::ICE,
            input_b: MaterialId::LAVA,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::WATER,
            output_b: MaterialId::STONE,
            probability: 0.4,
            energy_released: -80.0, // Endothermic
        });

        // ===== PHASE 5: NEW REACTIONS (20+) =====

        // === SMELTING REACTIONS ===

        // Iron Ore + Fire → Iron Ingot + Smoke (high temp required)
        self.register(Reaction {
            name: "smelt_iron".to_string(),
            input_a: MaterialId::IRON_ORE,
            input_b: MaterialId::FIRE,
            min_temp: Some(1200.0), // High temperature required
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::IRON_INGOT,
            output_b: MaterialId::SMOKE,
            probability: 0.05,
            energy_released: 10.0, // Exothermic
        });

        // Copper Ore + Fire → Copper Ingot + Smoke
        self.register(Reaction {
            name: "smelt_copper".to_string(),
            input_a: MaterialId::COPPER_ORE,
            input_b: MaterialId::FIRE,
            min_temp: Some(1000.0),
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::COPPER_INGOT,
            output_b: MaterialId::SMOKE,
            probability: 0.06,
            energy_released: 8.0,
        });

        // Gold Ore + Fire → Gold Ingot + Smoke
        self.register(Reaction {
            name: "smelt_gold".to_string(),
            input_a: MaterialId::GOLD_ORE,
            input_b: MaterialId::FIRE,
            min_temp: Some(1064.0),
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::GOLD_INGOT, // Fixed: was COPPER_INGOT
            output_b: MaterialId::SMOKE,
            probability: 0.05,
            energy_released: 5.0,
        });

        // Sand + Fire → Glass (very high temp)
        self.register(Reaction {
            name: "melt_sand_glass".to_string(),
            input_a: MaterialId::SAND,
            input_b: MaterialId::FIRE,
            min_temp: Some(1700.0),
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::GLASS,
            output_b: MaterialId::SMOKE,
            probability: 0.02,
            energy_released: 15.0,
        });

        // === COOKING/ORGANIC REACTIONS ===

        // Flesh + Fire → Ash + Smoke (cooking/burning)
        self.register(Reaction {
            name: "cook_flesh".to_string(),
            input_a: MaterialId::FLESH,
            input_b: MaterialId::FIRE,
            min_temp: Some(200.0),
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::ASH,
            output_b: MaterialId::SMOKE,
            probability: 0.08,
            energy_released: 20.0,
        });

        // Plant Matter + Fire → Ash + Smoke
        self.register(Reaction {
            name: "burn_plant".to_string(),
            input_a: MaterialId::PLANT_MATTER,
            input_b: MaterialId::FIRE,
            min_temp: Some(250.0),
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::ASH,
            output_b: MaterialId::SMOKE,
            probability: 0.07,
            energy_released: 12.0,
        });

        // Fruit + Fire → Ash + Steam (water content)
        self.register(Reaction {
            name: "burn_fruit".to_string(),
            input_a: MaterialId::FRUIT,
            input_b: MaterialId::FIRE,
            min_temp: Some(150.0),
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::ASH,
            output_b: MaterialId::STEAM,
            probability: 0.06,
            energy_released: 8.0,
        });

        // === EXPLOSIVE REACTIONS ===

        // Gunpowder + Fire → Smoke + Smoke (rapid explosion)
        self.register(Reaction {
            name: "explode_gunpowder".to_string(),
            input_a: MaterialId::GUNPOWDER,
            input_b: MaterialId::FIRE,
            min_temp: Some(150.0),
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::SMOKE,
            output_b: MaterialId::SMOKE,
            probability: 0.9,       // Very rapid
            energy_released: 100.0, // Highly exothermic
        });

        // === DECAY/DECOMPOSITION REACTIONS ===

        // Flesh + Water → Poison Gas + Poison Gas (decay in water)
        self.register(Reaction {
            name: "decay_flesh".to_string(),
            input_a: MaterialId::FLESH,
            input_b: MaterialId::WATER,
            min_temp: Some(15.0), // Room temp or above
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::POISON_GAS,
            output_b: MaterialId::POISON_GAS,
            probability: 0.001, // Very slow decay
            energy_released: -5.0,
        });

        // === GROWTH/LIFE REACTIONS ===

        // Plant Matter + Water → Plant Matter + Plant Matter (growth, requires light)
        // NOTE: Light check not implemented yet, will be added in Milestone 4
        self.register(Reaction {
            name: "grow_plant".to_string(),
            input_a: MaterialId::PLANT_MATTER,
            input_b: MaterialId::WATER,
            min_temp: Some(10.0),
            max_temp: Some(40.0),
            requires_contact: true,
            requires_light: Some(8), // Requires light >= 8 (not checked yet)
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::PLANT_MATTER,
            output_b: MaterialId::PLANT_MATTER,
            probability: 0.0005,   // Very slow growth
            energy_released: -3.0, // Endothermic (photosynthesis)
        });

        // Plant Matter + Fertilizer → Plant Matter + Dirt (fertilizer consumed)
        self.register(Reaction {
            name: "fertilize_plant".to_string(),
            input_a: MaterialId::PLANT_MATTER,
            input_b: MaterialId::FERTILIZER,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::PLANT_MATTER,
            output_b: MaterialId::DIRT,
            probability: 0.01,
            energy_released: 0.0,
        });

        // === COMPOSTING/RECYCLING ===

        // Ash + Water → Fertilizer + Air
        self.register(Reaction {
            name: "compost_ash".to_string(),
            input_a: MaterialId::ASH,
            input_b: MaterialId::WATER,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::FERTILIZER,
            output_b: MaterialId::AIR,
            probability: 0.005,
            energy_released: 0.0,
        });

        // === ADVANCED CHEMISTRY ===

        // Coal Ore + Acid → Gunpowder + Poison Gas (sulfur extraction)
        self.register(Reaction {
            name: "extract_sulfur".to_string(),
            input_a: MaterialId::COAL_ORE,
            input_b: MaterialId::ACID,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::GUNPOWDER,
            output_b: MaterialId::POISON_GAS,
            probability: 0.02,
            energy_released: -10.0,
        });

        // Acid + Bone → Air + Air (dissolves bone)
        self.register(Reaction {
            name: "dissolve_bone".to_string(),
            input_a: MaterialId::ACID,
            input_b: MaterialId::BONE,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::AIR,
            output_b: MaterialId::AIR,
            probability: 0.02,
            energy_released: 5.0,
        });

        // === ALLOY CREATION (Future: requires crafting system) ===
        // Copper + Iron → Bronze (would need crafting interface)
        // Iron + Coal → Steel (would need crafting interface)

        // === TEMPERATURE-BASED STATE CHANGES (already handled by material melting/freezing) ===
        // These are handled by the thermal system, not reactions

        // === GAS REACTIONS ===

        // Poison Gas + Water → Acid + Air (gas absorption)
        self.register(Reaction {
            name: "absorb_poison".to_string(),
            input_a: MaterialId::POISON_GAS,
            input_b: MaterialId::WATER,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::ACID,
            output_b: MaterialId::AIR,
            probability: 0.03,
            energy_released: 0.0,
        });

        // Steam + Cold Stone → Water + Stone (condensation)
        self.register(Reaction {
            name: "condense_steam".to_string(),
            input_a: MaterialId::STEAM,
            input_b: MaterialId::STONE,
            min_temp: None,
            max_temp: Some(80.0), // Cold stone
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::WATER,
            output_b: MaterialId::STONE,
            probability: 0.05,
            energy_released: 15.0, // Exothermic (releases latent heat)
        });

        // === CORROSION EXTENSIONS ===

        // Acid + Copper Ingot → Air + Poison Gas
        self.register(Reaction {
            name: "corrode_copper".to_string(),
            input_a: MaterialId::ACID,
            input_b: MaterialId::COPPER_INGOT,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::AIR,
            output_b: MaterialId::POISON_GAS,
            probability: 0.04,
            energy_released: 0.0,
        });

        // Acid + Iron Ingot → Air + Poison Gas
        self.register(Reaction {
            name: "corrode_iron".to_string(),
            input_a: MaterialId::ACID,
            input_b: MaterialId::IRON_ORE,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::AIR,
            output_b: MaterialId::POISON_GAS,
            probability: 0.03,
            energy_released: 0.0,
        });

        // === DIRT/SOIL REACTIONS ===

        // Dirt + Water → Sand + Water (erosion)
        self.register(Reaction {
            name: "erode_dirt".to_string(),
            input_a: MaterialId::DIRT,
            input_b: MaterialId::WATER,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::SAND,
            output_b: MaterialId::WATER,
            probability: 0.001, // Very slow
            energy_released: 0.0,
        });

        // Total: 5 original + 21 new = 26 reactions!

        // === ELECTRICAL REACTIONS ===

        // === ELECTRICAL REACTIONS ===

        // Spark + Flammable Material -> Fire
        // Note: C_4 and BOMB are excluded - they have specialized detonation reactions
        for mat_def in materials.all_materials() {
            if mat_def.flammable
                && mat_def.id != MaterialId::FIRE
                && mat_def.id != MaterialId::SPARK
                && mat_def.id != MaterialId::C_4
                && mat_def.id != MaterialId::BOMB
            {
                self.register(Reaction {
                    name: format!("spark_ignite_{}", mat_def.name),
                    input_a: MaterialId::SPARK,
                    input_b: mat_def.id,
                    min_temp: None,
                    max_temp: None,
                    requires_contact: true,
                    requires_light: None,
                    min_pressure: None,
                    catalyst: None,
                    output_a: MaterialId::AIR,  // Spark is consumed
                    output_b: MaterialId::FIRE, // Flammable material catches fire
                    probability: 0.8,
                    energy_released: 50.0, // Exothermic
                });
            }
        }

        // Spark + Water -> Steam
        self.register(Reaction {
            name: "spark_water_steam".to_string(),
            input_a: MaterialId::SPARK,
            input_b: MaterialId::WATER,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::AIR,
            output_b: MaterialId::STEAM,
            probability: 0.5,
            energy_released: -20.0, // Endothermic
        });

        // Thunder + Non-Conductor -> Air
        for mat_def in materials.all_materials() {
            // Thunder does not destroy itself, air, or laser (which has special handling)
            if !mat_def.conducts_electricity
                && mat_def.id != MaterialId::AIR
                && mat_def.id != MaterialId::THUNDER
                && mat_def.id != MaterialId::LASER
            {
                self.register(Reaction {
                    name: format!("thunder_destroy_{}", mat_def.name),
                    input_a: MaterialId::THUNDER,
                    input_b: mat_def.id,
                    min_temp: None,
                    max_temp: None,
                    requires_contact: true,
                    requires_light: None,
                    min_pressure: None,
                    catalyst: None,
                    output_a: MaterialId::AIR, // Thunder is consumed
                    output_b: MaterialId::AIR, // Non-conductor is destroyed
                    probability: 0.95,
                    energy_released: 200.0, // Highly exothermic
                });
            }
        }
        // Total: 5 original + 21 new + electrical = approximately 50+ reactions!

        // === PRESSURE REACTIONS ===

        // Nitro + Air (or anything) -> Smoke + Air (explosion when min_pressure is met)
        self.register(Reaction {
            name: "nitro_explosion".to_string(),
            input_a: MaterialId::NITRO,
            input_b: MaterialId::AIR, // Can explode in air
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: Some(20.0), // Explodes when pressure is high
            catalyst: None,
            output_a: MaterialId::SMOKE,
            output_b: MaterialId::AIR,
            probability: 1.0,       // Instant explosion
            energy_released: 500.0, // Massive energy release
        });

        // ===== WEEK 5: ADDITIONAL POWDER GAME REACTIONS =====

        // === C-4 DETONATION ===

        // C-4 + Spark -> Smoke + Fire (electrical detonation)
        self.register(Reaction {
            name: "c4_spark_detonate".to_string(),
            input_a: MaterialId::C_4,
            input_b: MaterialId::SPARK,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::SMOKE,
            output_b: MaterialId::FIRE,
            probability: 0.95,      // High probability with spark
            energy_released: 800.0, // Very powerful explosion
        });

        // C-4 + Fire -> Smoke + Fire (needs high temp to detonate)
        self.register(Reaction {
            name: "c4_fire_detonate".to_string(),
            input_a: MaterialId::C_4,
            input_b: MaterialId::FIRE,
            min_temp: Some(400.0), // Requires high temperature
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::SMOKE,
            output_b: MaterialId::FIRE,
            probability: 0.8,
            energy_released: 800.0,
        });

        // === BOMB EXPLOSION ===

        // Bomb + Fire -> Smoke + Fire (contact detonation)
        self.register(Reaction {
            name: "bomb_fire_detonate".to_string(),
            input_a: MaterialId::BOMB,
            input_b: MaterialId::FIRE,
            min_temp: None, // Low temp threshold
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::SMOKE,
            output_b: MaterialId::FIRE,
            probability: 0.9,
            energy_released: 400.0, // Medium explosion
        });

        // Bomb + Sand -> Smoke + Fire (impact detonation with powder)
        self.register(Reaction {
            name: "bomb_sand_impact".to_string(),
            input_a: MaterialId::BOMB,
            input_b: MaterialId::SAND,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: Some(5.0), // Needs some impact
            catalyst: None,
            output_a: MaterialId::SMOKE,
            output_b: MaterialId::FIRE,
            probability: 0.7,
            energy_released: 400.0,
        });

        // Bomb + Stone -> Smoke + Fire (impact detonation with solid)
        self.register(Reaction {
            name: "bomb_stone_impact".to_string(),
            input_a: MaterialId::BOMB,
            input_b: MaterialId::STONE,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: Some(3.0), // Lower threshold for hard impact
            catalyst: None,
            output_a: MaterialId::SMOKE,
            output_b: MaterialId::AIR,
            probability: 0.6,
            energy_released: 400.0,
        });

        // === MAGMA REACTIONS ===

        // Magma + Water -> Lava + Steam (rapid cooling)
        self.register(Reaction {
            name: "magma_water_cool".to_string(),
            input_a: MaterialId::MAGMA,
            input_b: MaterialId::WATER,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::LAVA,
            output_b: MaterialId::STEAM,
            probability: 0.5,        // Instant cooling
            energy_released: -200.0, // Very endothermic
        });

        // Magma + Ice -> Lava + Water
        self.register(Reaction {
            name: "magma_ice_melt".to_string(),
            input_a: MaterialId::MAGMA,
            input_b: MaterialId::ICE,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::LAVA,
            output_b: MaterialId::WATER,
            probability: 0.6,
            energy_released: -150.0,
        });

        // Magma + Wood -> Fire + Fire (instant ignition)
        self.register(Reaction {
            name: "magma_wood_ignite".to_string(),
            input_a: MaterialId::MAGMA,
            input_b: MaterialId::WOOD,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::FIRE,
            output_b: MaterialId::FIRE,
            probability: 0.95, // Almost instant
            energy_released: 100.0,
        });

        // Magma + Oil -> Fire + Fire (instant ignition)
        self.register(Reaction {
            name: "magma_oil_ignite".to_string(),
            input_a: MaterialId::MAGMA,
            input_b: MaterialId::OIL,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::FIRE,
            output_b: MaterialId::FIRE,
            probability: 0.98, // Even faster than wood
            energy_released: 150.0,
        });

        // === SALT REACTIONS ===

        // Salt + Water -> Seawater + Air (dissolution)
        self.register(Reaction {
            name: "salt_dissolve".to_string(),
            input_a: MaterialId::SALT,
            input_b: MaterialId::WATER,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::SEAWATER,
            output_b: MaterialId::AIR,
            probability: 0.1,      // Gradual dissolution
            energy_released: -5.0, // Slightly endothermic
        });

        // === SEAWATER REACTIONS ===

        // Seawater + Fire -> Steam + Salt (evaporation leaves salt)
        self.register(Reaction {
            name: "seawater_evaporate".to_string(),
            input_a: MaterialId::SEAWATER,
            input_b: MaterialId::FIRE,
            min_temp: Some(100.0),
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::STEAM,
            output_b: MaterialId::SALT,
            probability: 0.08,
            energy_released: 30.0,
        });

        // Seawater + Lava -> Steam + Salt (rapid evaporation)
        self.register(Reaction {
            name: "seawater_lava_evaporate".to_string(),
            input_a: MaterialId::SEAWATER,
            input_b: MaterialId::LAVA,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::STEAM,
            output_b: MaterialId::STONE, // Lava cools + salt deposits
            probability: 0.25,
            energy_released: -80.0,
        });

        // === SOAPY WATER REACTIONS ===

        // Soapy Water + Air -> Bubble + Soapy Water (bubble creation)
        self.register(Reaction {
            name: "soapy_bubble_create".to_string(),
            input_a: MaterialId::SOAPY_WATER,
            input_b: MaterialId::AIR,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::SOAPY_WATER, // Soapy water preserved
            output_b: MaterialId::BUBBLE,      // Air becomes bubble
            probability: 0.01,                 // Low probability - occasional bubbles
            energy_released: 0.0,
        });

        // Soapy Water + Pressure -> Bubble + Soapy Water (agitated bubbles)
        self.register(Reaction {
            name: "soapy_pressure_bubble".to_string(),
            input_a: MaterialId::SOAPY_WATER,
            input_b: MaterialId::AIR,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: Some(10.0), // Higher pressure creates more bubbles
            catalyst: None,
            output_a: MaterialId::SOAPY_WATER,
            output_b: MaterialId::BUBBLE,
            probability: 0.2, // Much higher with pressure
            energy_released: 0.0,
        });

        // === BUBBLE REACTIONS ===

        // Bubble + Fire -> Air + Air (pops in heat)
        self.register(Reaction {
            name: "bubble_fire_pop".to_string(),
            input_a: MaterialId::BUBBLE,
            input_b: MaterialId::FIRE,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::AIR,
            output_b: MaterialId::AIR,
            probability: 0.99, // Instant pop
            energy_released: 0.0,
        });

        // Bubble + Stone -> Air + Stone (pops on solid contact)
        self.register(Reaction {
            name: "bubble_stone_pop".to_string(),
            input_a: MaterialId::BUBBLE,
            input_b: MaterialId::STONE,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::AIR,
            output_b: MaterialId::STONE,
            probability: 0.5, // 50% chance to pop on contact
            energy_released: 0.0,
        });

        // Bubble + Metal -> Air + Metal (pops on metal)
        self.register(Reaction {
            name: "bubble_metal_pop".to_string(),
            input_a: MaterialId::BUBBLE,
            input_b: MaterialId::METAL,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::AIR,
            output_b: MaterialId::METAL,
            probability: 0.6,
            energy_released: 0.0,
        });

        // Bubble + Glass -> Air + Glass (pops on glass - sharp)
        self.register(Reaction {
            name: "bubble_glass_pop".to_string(),
            input_a: MaterialId::BUBBLE,
            input_b: MaterialId::GLASS,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::AIR,
            output_b: MaterialId::GLASS,
            probability: 0.8, // Glass is sharp - higher pop rate
            energy_released: 0.0,
        });

        // === MERCURY REACTIONS ===

        // Mercury + Spark -> Spark + Mercury (conducts electricity)
        // Note: Mercury just passes spark through, handled by conductivity

        // Mercury + Fire -> Poison Gas + Air (vaporization)
        self.register(Reaction {
            name: "mercury_vaporize".to_string(),
            input_a: MaterialId::MERCURY,
            input_b: MaterialId::FIRE,
            min_temp: Some(357.0), // Mercury boiling point
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: None,
            output_a: MaterialId::POISON_GAS,
            output_b: MaterialId::FIRE,
            probability: 0.1,
            energy_released: 20.0,
        });
    }

    /// Register a new reaction
    fn register(&mut self, reaction: Reaction) {
        // Normalize material order (lower ID first) for consistent HashMap key
        let key = if reaction.input_a <= reaction.input_b {
            (reaction.input_a, reaction.input_b)
        } else {
            (reaction.input_b, reaction.input_a)
        };

        // Add to HashMap (may have multiple reactions for same material pair)
        self.reactions.entry(key).or_default().push(reaction);
    }

    /// Find a matching reaction between two materials at given conditions
    ///
    /// Returns the first matching reaction, or None if no reaction possible
    /// O(1) HashMap lookup + O(k) where k = reactions for this material pair (typically 1-3)
    ///
    /// # Parameters
    /// - `mat_a`, `mat_b`: Materials in contact
    /// - `temp`: Temperature at the reaction site
    /// - `light_level`: Light level (0-15)
    /// - `pressure`: Gas pressure at the site
    /// - `neighbor_materials`: Materials in 8-connected neighborhood (for catalyst check)
    pub fn find_reaction(
        &self,
        mat_a: u16,
        mat_b: u16,
        temp: f32,
        light_level: u8,
        pressure: f32,
        neighbor_materials: &[u16],
    ) -> Option<&Reaction> {
        // Normalize material order for HashMap key
        let key = if mat_a <= mat_b {
            (mat_a, mat_b)
        } else {
            (mat_b, mat_a)
        };

        // O(1) HashMap lookup
        let reactions = self.reactions.get(&key)?;

        // Check each reaction for this material pair
        for reaction in reactions {
            // Check temperature conditions
            if let Some(min_t) = reaction.min_temp
                && temp < min_t
            {
                continue;
            }
            if let Some(max_t) = reaction.max_temp
                && temp > max_t
            {
                continue;
            }

            // Check light condition
            if let Some(min_light) = reaction.requires_light
                && light_level < min_light
            {
                continue; // Insufficient light
            }

            // Check pressure condition
            if let Some(min_p) = reaction.min_pressure
                && pressure < min_p
            {
                continue; // Insufficient pressure
            }

            // Check catalyst condition
            if let Some(catalyst_mat) = reaction.catalyst
                && !neighbor_materials.contains(&catalyst_mat)
            {
                continue; // Required catalyst not present
            }

            // All conditions met!
            return Some(reaction);
        }

        None
    }

    /// Get output materials for a reaction, accounting for which input is which
    ///
    /// Returns (output_for_mat_a, output_for_mat_b)
    pub fn get_outputs(&self, reaction: &Reaction, mat_a: u16, mat_b: u16) -> (u16, u16) {
        // If materials match in original order, use outputs as-is
        if reaction.input_a == mat_a && reaction.input_b == mat_b {
            (reaction.output_a, reaction.output_b)
        } else {
            // Materials are swapped, swap outputs too
            (reaction.output_b, reaction.output_a)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::Materials;

    #[test]
    fn test_find_reaction_forward() {
        let materials = Materials::new();
        let registry = ReactionRegistry::new(&materials);

        // Water + Lava should find reaction
        let reaction =
            registry.find_reaction(MaterialId::WATER, MaterialId::LAVA, 20.0, 0, 1.0, &[]);
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().name, "water_lava_steam");
    }

    #[test]
    fn test_find_reaction_backward() {
        let materials = Materials::new();
        let registry = ReactionRegistry::new(&materials);

        // Lava + Water should also find reaction (order doesn't matter)
        let reaction =
            registry.find_reaction(MaterialId::LAVA, MaterialId::WATER, 20.0, 0, 1.0, &[]);
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().name, "water_lava_steam");
    }

    #[test]
    fn test_no_reaction() {
        let materials = Materials::new();
        let registry = ReactionRegistry::new(&materials);

        // Sand + Water has no reaction
        let reaction =
            registry.find_reaction(MaterialId::SAND, MaterialId::WATER, 20.0, 0, 1.0, &[]);
        assert!(reaction.is_none());
    }

    #[test]
    fn test_get_outputs() {
        let materials = Materials::new();
        let registry = ReactionRegistry::new(&materials);

        let reaction = registry
            .find_reaction(MaterialId::WATER, MaterialId::LAVA, 20.0, 0, 1.0, &[])
            .unwrap();

        // Water + Lava
        let (out_a, out_b) = registry.get_outputs(reaction, MaterialId::WATER, MaterialId::LAVA);
        assert_eq!(out_a, MaterialId::STEAM); // Water → Steam
        assert_eq!(out_b, MaterialId::STONE); // Lava → Stone

        // Lava + Water (swapped)
        let (out_a, out_b) = registry.get_outputs(reaction, MaterialId::LAVA, MaterialId::WATER);
        assert_eq!(out_a, MaterialId::STONE); // Lava → Stone
        assert_eq!(out_b, MaterialId::STEAM); // Water → Steam
    }

    #[test]
    fn test_catalyst_required() {
        let materials = Materials::new();
        let mut registry = ReactionRegistry::new(&materials); // Using new constructor

        // Create a test reaction that requires a catalyst
        registry.register(Reaction {
            name: "test_catalyst".to_string(),
            input_a: MaterialId::STONE,
            input_b: MaterialId::WATER,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: None,
            catalyst: Some(MaterialId::FIRE), // Requires fire as catalyst
            output_a: MaterialId::STEAM,
            output_b: MaterialId::AIR,
            probability: 1.0,
            energy_released: 0.0,
        });

        // Without catalyst - no reaction
        let reaction = registry.find_reaction(
            MaterialId::STONE,
            MaterialId::WATER,
            20.0,
            0,
            1.0,
            &[MaterialId::AIR, MaterialId::SAND], // No fire
        );
        assert!(reaction.is_none());

        // With catalyst - reaction occurs
        let reaction = registry.find_reaction(
            MaterialId::STONE,
            MaterialId::WATER,
            20.0,
            0,
            1.0,
            &[MaterialId::AIR, MaterialId::FIRE, MaterialId::SAND], // Fire present
        );
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().name, "test_catalyst");
    }

    #[test]
    fn test_pressure_required() {
        let materials = Materials::new();
        let mut registry = ReactionRegistry::new(&materials); // Using new constructor

        // Create a test reaction that requires minimum pressure
        registry.register(Reaction {
            name: "test_pressure".to_string(),
            input_a: MaterialId::STEAM,
            input_b: MaterialId::WATER,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            requires_light: None,
            min_pressure: Some(5.0), // Requires pressure >= 5.0
            catalyst: None,
            output_a: MaterialId::AIR,
            output_b: MaterialId::AIR,
            probability: 1.0,
            energy_released: 0.0,
        });

        // Low pressure - no reaction
        let reaction = registry.find_reaction(
            MaterialId::STEAM,
            MaterialId::WATER,
            20.0,
            0,
            2.0, // Below threshold
            &[],
        );
        assert!(reaction.is_none());

        // High pressure - reaction occurs
        let reaction = registry.find_reaction(
            MaterialId::STEAM,
            MaterialId::WATER,
            20.0,
            0,
            10.0, // Above threshold
            &[],
        );
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().name, "test_pressure");
    }

    // ===== WEEK 5 REACTION TESTS =====

    #[test]
    fn test_c4_detonation() {
        let materials = Materials::new();
        let registry = ReactionRegistry::new(&materials);

        // C-4 + Spark should detonate
        let reaction =
            registry.find_reaction(MaterialId::C_4, MaterialId::SPARK, 20.0, 0, 1.0, &[]);
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().name, "c4_spark_detonate");

        // C-4 + Fire at low temp should NOT detonate (needs 400°C)
        let reaction = registry.find_reaction(
            MaterialId::C_4,
            MaterialId::FIRE,
            100.0, // Below 400°C
            0,
            1.0,
            &[],
        );
        assert!(reaction.is_none());

        // C-4 + Fire at high temp should detonate
        let reaction = registry.find_reaction(
            MaterialId::C_4,
            MaterialId::FIRE,
            500.0, // Above 400°C
            0,
            1.0,
            &[],
        );
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().name, "c4_fire_detonate");
    }

    #[test]
    fn test_bomb_detonation() {
        let materials = Materials::new();
        let registry = ReactionRegistry::new(&materials);

        // Bomb + Fire should detonate
        let reaction =
            registry.find_reaction(MaterialId::BOMB, MaterialId::FIRE, 20.0, 0, 1.0, &[]);
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().name, "bomb_fire_detonate");
    }

    #[test]
    fn test_magma_reactions() {
        let materials = Materials::new();
        let registry = ReactionRegistry::new(&materials);

        // Magma + Water -> Lava + Steam
        let reaction =
            registry.find_reaction(MaterialId::MAGMA, MaterialId::WATER, 20.0, 0, 1.0, &[]);
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().name, "magma_water_cool");

        // Magma + Wood -> Fire + Fire (instant ignition)
        let reaction =
            registry.find_reaction(MaterialId::MAGMA, MaterialId::WOOD, 20.0, 0, 1.0, &[]);
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().name, "magma_wood_ignite");
    }

    #[test]
    fn test_salt_dissolution() {
        let materials = Materials::new();
        let registry = ReactionRegistry::new(&materials);

        // Salt + Water -> Seawater + Air
        let reaction =
            registry.find_reaction(MaterialId::SALT, MaterialId::WATER, 20.0, 0, 1.0, &[]);
        assert!(reaction.is_some());
        let r = reaction.unwrap();
        assert_eq!(r.name, "salt_dissolve");
        assert_eq!(r.output_a, MaterialId::SEAWATER);
    }

    #[test]
    fn test_seawater_evaporation() {
        let materials = Materials::new();
        let registry = ReactionRegistry::new(&materials);

        // Seawater + Fire at low temp - no evaporation
        let reaction = registry.find_reaction(
            MaterialId::SEAWATER,
            MaterialId::FIRE,
            50.0, // Below 100°C
            0,
            1.0,
            &[],
        );
        assert!(reaction.is_none());

        // Seawater + Fire at high temp -> Steam + Salt
        let reaction = registry.find_reaction(
            MaterialId::SEAWATER,
            MaterialId::FIRE,
            150.0, // Above 100°C
            0,
            1.0,
            &[],
        );
        assert!(reaction.is_some());
        let r = reaction.unwrap();
        assert_eq!(r.name, "seawater_evaporate");
        assert_eq!(r.output_a, MaterialId::STEAM);
        assert_eq!(r.output_b, MaterialId::SALT);
    }

    #[test]
    fn test_soapy_bubble_creation() {
        let materials = Materials::new();
        let registry = ReactionRegistry::new(&materials);

        // Soapy Water + Air -> Bubble (low probability)
        let reaction =
            registry.find_reaction(MaterialId::SOAPY_WATER, MaterialId::AIR, 20.0, 0, 1.0, &[]);
        assert!(reaction.is_some());
        let r = reaction.unwrap();
        assert_eq!(r.name, "soapy_bubble_create");
        assert_eq!(r.output_b, MaterialId::BUBBLE);
    }

    #[test]
    fn test_bubble_popping() {
        let materials = Materials::new();
        let registry = ReactionRegistry::new(&materials);

        // Bubble + Fire -> Air + Air
        let reaction =
            registry.find_reaction(MaterialId::BUBBLE, MaterialId::FIRE, 20.0, 0, 1.0, &[]);
        assert!(reaction.is_some());
        let r = reaction.unwrap();
        assert_eq!(r.name, "bubble_fire_pop");
        assert_eq!(r.output_a, MaterialId::AIR);

        // Bubble + Glass -> Air + Glass (sharp surface)
        let reaction =
            registry.find_reaction(MaterialId::BUBBLE, MaterialId::GLASS, 20.0, 0, 1.0, &[]);
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().name, "bubble_glass_pop");
    }

    #[test]
    fn test_mercury_vaporization() {
        let materials = Materials::new();
        let registry = ReactionRegistry::new(&materials);

        // Mercury + Fire at low temp - no vaporization
        let reaction = registry.find_reaction(
            MaterialId::MERCURY,
            MaterialId::FIRE,
            100.0, // Below 357°C
            0,
            1.0,
            &[],
        );
        assert!(reaction.is_none());

        // Mercury + Fire at high temp -> Poison Gas
        let reaction = registry.find_reaction(
            MaterialId::MERCURY,
            MaterialId::FIRE,
            400.0, // Above 357°C
            0,
            1.0,
            &[],
        );
        assert!(reaction.is_some());
        let r = reaction.unwrap();
        assert_eq!(r.name, "mercury_vaporize");
        assert_eq!(r.output_a, MaterialId::POISON_GAS);
    }
}
