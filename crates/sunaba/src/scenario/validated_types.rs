//! Validated types for scenario definitions
//!
//! These types provide parse-time validation for scenario values,
//! catching errors early with helpful error messages.

use anyhow::{Result, bail};
use serde::{Deserialize, Deserializer, Serialize};

// ============================================================================
// High-priority validation (safety - prevents panics/undefined behavior)
// ============================================================================

/// Validated material ID (0-37)
///
/// Ensures material IDs are within valid range to prevent undefined behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedMaterialId(u16);

impl ValidatedMaterialId {
    pub const MAX: u16 = 37; // OBSIDIAN is highest defined material

    pub fn new(id: u16) -> Result<Self> {
        if id > Self::MAX {
            bail!(
                "Invalid material ID: {}. Valid range: 0-{} (see MaterialId constants in sunaba-simulation)",
                id,
                Self::MAX
            );
        }
        Ok(Self(id))
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ValidatedMaterialId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = u16::deserialize(deserializer)?;
        Self::new(id).map_err(serde::de::Error::custom)
    }
}

impl Serialize for ValidatedMaterialId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u16(self.0)
    }
}

/// Validated inventory slot index (0-49)
///
/// Ensures slot indices are within inventory bounds to prevent panics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedSlotIndex(usize);

impl ValidatedSlotIndex {
    pub const MAX: usize = 49; // 0-49 (50 total slots)

    pub fn new(slot: usize) -> Result<Self> {
        if slot > Self::MAX {
            bail!(
                "Invalid inventory slot: {}. Valid range: 0-{} (inventory has 50 slots)",
                slot,
                Self::MAX
            );
        }
        Ok(Self(slot))
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

impl<'de> Deserialize<'de> for ValidatedSlotIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let slot = usize::deserialize(deserializer)?;
        Self::new(slot).map_err(serde::de::Error::custom)
    }
}

impl Serialize for ValidatedSlotIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.0 as u64)
    }
}

/// Validated radius (1-1000)
///
/// Ensures radius is positive and reasonable to prevent infinite loops/panics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedRadius(u32);

impl ValidatedRadius {
    pub const MIN: u32 = 1;
    pub const MAX: u32 = 1000;

    pub fn new(radius: u32) -> Result<Self> {
        if radius < Self::MIN {
            bail!("Radius must be at least {} (got: {})", Self::MIN, radius);
        }
        if radius > Self::MAX {
            bail!(
                "Radius too large: {}. Maximum: {} (to prevent performance issues)",
                radius,
                Self::MAX
            );
        }
        Ok(Self(radius))
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ValidatedRadius {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let radius = u32::deserialize(deserializer)?;
        Self::new(radius).map_err(serde::de::Error::custom)
    }
}

impl Serialize for ValidatedRadius {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.0)
    }
}

// ============================================================================
// Medium-priority validation (correctness)
// ============================================================================

/// Validated health value (0.0-100.0, finite)
///
/// Ensures health values are valid and won't cause physics bugs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValidatedHealth(f32);

impl ValidatedHealth {
    pub const MAX: f32 = 100.0;

    pub fn new(health: f32) -> Result<Self> {
        if !health.is_finite() {
            bail!(
                "Health must be a finite number (got: {}). NaN and infinity are not allowed.",
                health
            );
        }
        if health < 0.0 {
            bail!("Health cannot be negative (got: {})", health);
        }
        if health > Self::MAX {
            bail!("Health too high: {}. Maximum: {}", health, Self::MAX);
        }
        Ok(Self(health))
    }

    pub fn get(&self) -> f32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ValidatedHealth {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let health = f32::deserialize(deserializer)?;
        Self::new(health).map_err(serde::de::Error::custom)
    }
}

impl Serialize for ValidatedHealth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f32(self.0)
    }
}

/// Validated hunger value (0.0-100.0, finite)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValidatedHunger(f32);

impl ValidatedHunger {
    pub const MAX: f32 = 100.0;

    pub fn new(hunger: f32) -> Result<Self> {
        if !hunger.is_finite() {
            bail!(
                "Hunger must be a finite number (got: {}). NaN and infinity are not allowed.",
                hunger
            );
        }
        if hunger < 0.0 {
            bail!("Hunger cannot be negative (got: {})", hunger);
        }
        if hunger > Self::MAX {
            bail!("Hunger too high: {}. Maximum: {}", hunger, Self::MAX);
        }
        Ok(Self(hunger))
    }

    pub fn get(&self) -> f32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ValidatedHunger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hunger = f32::deserialize(deserializer)?;
        Self::new(hunger).map_err(serde::de::Error::custom)
    }
}

impl Serialize for ValidatedHunger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f32(self.0)
    }
}

// ============================================================================
// String enum validation
// ============================================================================

/// Validated simulated key input
///
/// Only allows known keyboard keys to prevent runtime errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub enum SimulatedKey {
    W,
    A,
    S,
    D,
    Space,
}

impl TryFrom<String> for SimulatedKey {
    type Error = String;

    fn try_from(s: String) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "w" => Ok(Self::W),
            "a" => Ok(Self::A),
            "s" => Ok(Self::S),
            "d" => Ok(Self::D),
            "space" => Ok(Self::Space),
            _ => Err(format!(
                "Invalid key: '{}'. Valid keys: w, a, s, d, space (case-insensitive)",
                s
            )),
        }
    }
}

impl SimulatedKey {
    /// Convert to string representation for executor
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::W => "w",
            Self::A => "a",
            Self::S => "s",
            Self::D => "d",
            Self::Space => "space",
        }
    }
}

impl Serialize for SimulatedKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Validated creature archetype
///
/// Only allows known creature types to prevent runtime errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub enum CreatureArchetype {
    Spider,
    Snake,
    Worm,
    Flyer,
}

impl TryFrom<String> for CreatureArchetype {
    type Error = String;

    fn try_from(s: String) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "spider" => Ok(Self::Spider),
            "snake" => Ok(Self::Snake),
            "worm" => Ok(Self::Worm),
            "flyer" => Ok(Self::Flyer),
            "evolved" => Err("Cannot spawn 'evolved' creatures directly in scenarios. \
                 Use the training system to evolve creatures, then load saved genomes."
                .to_string()),
            _ => Err(format!(
                "Invalid creature type: '{}'. Valid types: spider, snake, worm, flyer (case-insensitive)",
                s
            )),
        }
    }
}

impl CreatureArchetype {
    /// Convert to string representation for executor
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Spider => "spider",
            Self::Snake => "snake",
            Self::Worm => "worm",
            Self::Flyer => "flyer",
        }
    }
}

impl Serialize for CreatureArchetype {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Validated mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    // Middle is intentionally excluded (not supported in scenarios)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_id_validation() {
        assert!(ValidatedMaterialId::new(0).is_ok()); // AIR
        assert!(ValidatedMaterialId::new(37).is_ok()); // OBSIDIAN (max)
        assert!(ValidatedMaterialId::new(38).is_err()); // Out of range
        assert!(ValidatedMaterialId::new(999).is_err()); // Way out of range
    }

    #[test]
    fn test_slot_index_validation() {
        assert!(ValidatedSlotIndex::new(0).is_ok());
        assert!(ValidatedSlotIndex::new(49).is_ok()); // Max slot
        assert!(ValidatedSlotIndex::new(50).is_err()); // Out of range
        assert!(ValidatedSlotIndex::new(999).is_err());
    }

    #[test]
    fn test_radius_validation() {
        assert!(ValidatedRadius::new(0).is_err()); // Too small
        assert!(ValidatedRadius::new(1).is_ok()); // Min
        assert!(ValidatedRadius::new(100).is_ok()); // Normal
        assert!(ValidatedRadius::new(1000).is_ok()); // Max
        assert!(ValidatedRadius::new(1001).is_err()); // Too large
    }

    #[test]
    fn test_health_validation() {
        assert!(ValidatedHealth::new(-1.0).is_err()); // Negative
        assert!(ValidatedHealth::new(0.0).is_ok());
        assert!(ValidatedHealth::new(50.0).is_ok());
        assert!(ValidatedHealth::new(100.0).is_ok()); // Max
        assert!(ValidatedHealth::new(101.0).is_err()); // Too high
        assert!(ValidatedHealth::new(f32::NAN).is_err()); // NaN
        assert!(ValidatedHealth::new(f32::INFINITY).is_err()); // Infinity
    }

    #[test]
    fn test_simulated_key_parsing() {
        assert_eq!(
            SimulatedKey::try_from("w".to_string()).unwrap(),
            SimulatedKey::W
        );
        assert_eq!(
            SimulatedKey::try_from("W".to_string()).unwrap(),
            SimulatedKey::W
        ); // Case-insensitive
        assert_eq!(
            SimulatedKey::try_from("space".to_string()).unwrap(),
            SimulatedKey::Space
        );
        assert!(SimulatedKey::try_from("invalid".to_string()).is_err());
    }

    #[test]
    fn test_creature_archetype_parsing() {
        assert_eq!(
            CreatureArchetype::try_from("spider".to_string()).unwrap(),
            CreatureArchetype::Spider
        );
        assert_eq!(
            CreatureArchetype::try_from("SNAKE".to_string()).unwrap(),
            CreatureArchetype::Snake
        ); // Case-insensitive
        assert!(CreatureArchetype::try_from("evolved".to_string()).is_err()); // Explicitly rejected
        assert!(CreatureArchetype::try_from("invalid".to_string()).is_err());
    }
}
