//! Chemical reaction system
//!
//! Handles interactions between different materials when they come into contact.
//! Examples: water + lava → steam + stone, acid + metal → air + air (corrosion)

use serde::{Serialize, Deserialize};
use crate::simulation::MaterialId;

/// Definition of a chemical reaction between two materials
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reaction {
    /// Human-readable name
    pub name: String,

    // Input materials
    pub input_a: u16,
    pub input_b: u16,

    // Conditions for reaction to occur
    pub min_temp: Option<f32>,
    pub max_temp: Option<f32>,
    pub requires_contact: bool,

    // Output materials (what each input becomes)
    pub output_a: u16,
    pub output_b: u16,

    /// Probability per frame when conditions are met (0.0 - 1.0)
    /// Lower values make reactions gradual rather than instant
    pub probability: f32,
}

/// Registry of all possible reactions
pub struct ReactionRegistry {
    reactions: Vec<Reaction>,
}

impl ReactionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            reactions: Vec::new(),
        };
        registry.register_default_reactions();
        registry
    }

    /// Register all default reactions
    fn register_default_reactions(&mut self) {
        // Water + Lava → Steam + Stone
        // When water touches hot lava, it rapidly boils to steam and cools the lava to stone
        self.register(Reaction {
            name: "water_lava_steam".to_string(),
            input_a: MaterialId::WATER,
            input_b: MaterialId::LAVA,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            output_a: MaterialId::STEAM,
            output_b: MaterialId::STONE,
            probability: 0.3, // 30% chance per contact per frame - makes it gradual
        });

        // Acid + Metal → Air + Air (corrosion)
        // Acid dissolves metal relatively quickly
        self.register(Reaction {
            name: "acid_metal_corrode".to_string(),
            input_a: MaterialId::ACID,
            input_b: MaterialId::METAL,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            output_a: MaterialId::AIR,
            output_b: MaterialId::AIR,
            probability: 0.05, // Slower corrosion
        });

        // Acid + Stone → Acid + Air (slower corrosion)
        // Acid also dissolves stone, but keeps the acid (only consumes the stone)
        self.register(Reaction {
            name: "acid_stone_corrode".to_string(),
            input_a: MaterialId::ACID,
            input_b: MaterialId::STONE,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            output_a: MaterialId::ACID,  // Acid remains
            output_b: MaterialId::AIR,   // Stone dissolved
            probability: 0.01, // Very slow
        });

        // Acid + Wood → Acid + Air
        // Acid dissolves wood
        self.register(Reaction {
            name: "acid_wood_corrode".to_string(),
            input_a: MaterialId::ACID,
            input_b: MaterialId::WOOD,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            output_a: MaterialId::ACID,
            output_b: MaterialId::AIR,
            probability: 0.03,
        });

        // Ice + Lava → Water + Stone
        // When ice touches lava, it melts and cools the lava
        self.register(Reaction {
            name: "ice_lava_cool".to_string(),
            input_a: MaterialId::ICE,
            input_b: MaterialId::LAVA,
            min_temp: None,
            max_temp: None,
            requires_contact: true,
            output_a: MaterialId::WATER,
            output_b: MaterialId::STONE,
            probability: 0.4, // Fairly rapid
        });
    }

    /// Register a new reaction
    fn register(&mut self, reaction: Reaction) {
        self.reactions.push(reaction);
    }

    /// Find a matching reaction between two materials at given conditions
    ///
    /// Returns the first matching reaction, or None if no reaction possible
    pub fn find_reaction(&self, mat_a: u16, mat_b: u16, temp: f32) -> Option<&Reaction> {
        for reaction in &self.reactions {
            // Check material match (bidirectional - order doesn't matter)
            let materials_match =
                (reaction.input_a == mat_a && reaction.input_b == mat_b) ||
                (reaction.input_a == mat_b && reaction.input_b == mat_a);

            if !materials_match {
                continue;
            }

            // Check temperature conditions
            if let Some(min_t) = reaction.min_temp {
                if temp < min_t {
                    continue;
                }
            }
            if let Some(max_t) = reaction.max_temp {
                if temp > max_t {
                    continue;
                }
            }

            // Reaction found!
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

impl Default for ReactionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_reaction_forward() {
        let registry = ReactionRegistry::new();

        // Water + Lava should find reaction
        let reaction = registry.find_reaction(MaterialId::WATER, MaterialId::LAVA, 20.0);
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().name, "water_lava_steam");
    }

    #[test]
    fn test_find_reaction_backward() {
        let registry = ReactionRegistry::new();

        // Lava + Water should also find reaction (order doesn't matter)
        let reaction = registry.find_reaction(MaterialId::LAVA, MaterialId::WATER, 20.0);
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().name, "water_lava_steam");
    }

    #[test]
    fn test_no_reaction() {
        let registry = ReactionRegistry::new();

        // Sand + Water has no reaction
        let reaction = registry.find_reaction(MaterialId::SAND, MaterialId::WATER, 20.0);
        assert!(reaction.is_none());
    }

    #[test]
    fn test_get_outputs() {
        let registry = ReactionRegistry::new();

        let reaction = registry.find_reaction(MaterialId::WATER, MaterialId::LAVA, 20.0).unwrap();

        // Water + Lava
        let (out_a, out_b) = registry.get_outputs(reaction, MaterialId::WATER, MaterialId::LAVA);
        assert_eq!(out_a, MaterialId::STEAM);  // Water → Steam
        assert_eq!(out_b, MaterialId::STONE);  // Lava → Stone

        // Lava + Water (swapped)
        let (out_a, out_b) = registry.get_outputs(reaction, MaterialId::LAVA, MaterialId::WATER);
        assert_eq!(out_a, MaterialId::STONE);  // Lava → Stone
        assert_eq!(out_b, MaterialId::STEAM);  // Water → Steam
    }
}
