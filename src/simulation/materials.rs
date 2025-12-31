//! Material definitions and registry

use serde::{Serialize, Deserialize};

/// Built-in material IDs
pub struct MaterialId;

impl MaterialId {
    pub const AIR: u16 = 0;
    pub const STONE: u16 = 1;
    pub const SAND: u16 = 2;
    pub const WATER: u16 = 3;
    pub const WOOD: u16 = 4;
    pub const FIRE: u16 = 5;
    pub const SMOKE: u16 = 6;
    pub const STEAM: u16 = 7;
    pub const LAVA: u16 = 8;
    pub const OIL: u16 = 9;
    pub const ACID: u16 = 10;
    pub const ICE: u16 = 11;
    pub const GLASS: u16 = 12;
    pub const METAL: u16 = 13;
    pub const BEDROCK: u16 = 14;
}

/// How a material behaves physically
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialType {
    /// Doesn't move (stone, wood, metal)
    Solid,
    /// Falls, piles up (sand, gravel, ash)
    Powder,
    /// Flows, seeks level (water, oil, lava)
    Liquid,
    /// Rises, disperses (steam, smoke)
    Gas,
}

/// Definition of a material's properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialDef {
    pub id: u16,
    pub name: String,
    pub material_type: MaterialType,
    
    /// Base color (RGBA)
    pub color: [u8; 4],
    
    /// Density (g/cm³) - affects sinking/floating
    pub density: f32,
    
    // Physical properties
    /// Mining/breaking resistance (None = unbreakable)
    pub hardness: Option<u8>,
    /// Sliding coefficient (powders)
    pub friction: f32,
    /// Flow speed (liquids)
    pub viscosity: f32,
    
    // Thermal properties
    /// Temperature at which this melts (Celsius)
    pub melting_point: Option<f32>,
    /// Temperature at which this boils/evaporates
    pub boiling_point: Option<f32>,
    /// Temperature at which this freezes
    pub freezing_point: Option<f32>,
    /// Temperature at which this ignites
    pub ignition_temp: Option<f32>,
    /// Heat conductivity (0.0 - 1.0)
    pub heat_conductivity: f32,
    
    // State transitions
    /// What this becomes when melted
    pub melts_to: Option<u16>,
    /// What this becomes when boiled
    pub boils_to: Option<u16>,
    /// What this becomes when frozen
    pub freezes_to: Option<u16>,
    /// What this becomes when burned
    pub burns_to: Option<u16>,
    /// How fast this burns (0.0 - 1.0)
    pub burn_rate: f32,
    
    // Flags
    pub flammable: bool,
    /// Can support other solid pixels
    pub structural: bool,
    pub conducts_electricity: bool,
}

impl Default for MaterialDef {
    fn default() -> Self {
        Self {
            id: 0,
            name: "unknown".to_string(),
            material_type: MaterialType::Solid,
            color: [255, 0, 255, 255], // Magenta for missing materials
            density: 1.0,
            hardness: Some(1),
            friction: 0.5,
            viscosity: 0.5,
            melting_point: None,
            boiling_point: None,
            freezing_point: None,
            ignition_temp: None,
            heat_conductivity: 0.5,
            melts_to: None,
            boils_to: None,
            freezes_to: None,
            burns_to: None,
            burn_rate: 0.0,
            flammable: false,
            structural: false,
            conducts_electricity: false,
        }
    }
}

/// Registry of all materials
pub struct Materials {
    materials: Vec<MaterialDef>,
}

impl Materials {
    pub fn new() -> Self {
        let mut materials = Self {
            materials: Vec::new(),
        };
        materials.register_defaults();
        materials
    }
    
    fn register_defaults(&mut self) {
        // Air (empty space)
        self.register(MaterialDef {
            id: MaterialId::AIR,
            name: "air".to_string(),
            material_type: MaterialType::Gas,
            color: [0, 0, 0, 0], // Transparent
            density: 0.001,
            hardness: None,
            ..Default::default()
        });
        
        // Stone
        self.register(MaterialDef {
            id: MaterialId::STONE,
            name: "stone".to_string(),
            material_type: MaterialType::Solid,
            color: [128, 128, 128, 255],
            density: 2.5,
            hardness: Some(5),
            structural: true,
            melting_point: Some(1200.0),
            melts_to: Some(MaterialId::LAVA),
            ..Default::default()
        });
        
        // Sand
        self.register(MaterialDef {
            id: MaterialId::SAND,
            name: "sand".to_string(),
            material_type: MaterialType::Powder,
            color: [194, 178, 128, 255],
            density: 1.5,
            hardness: Some(1),
            friction: 0.3,
            melting_point: Some(1700.0),
            melts_to: Some(MaterialId::GLASS),
            ..Default::default()
        });
        
        // Water
        self.register(MaterialDef {
            id: MaterialId::WATER,
            name: "water".to_string(),
            material_type: MaterialType::Liquid,
            color: [64, 164, 223, 200],
            density: 1.0,
            hardness: None,
            viscosity: 0.1,
            boiling_point: Some(100.0),
            boils_to: Some(MaterialId::STEAM),
            freezing_point: Some(0.0),
            freezes_to: Some(MaterialId::ICE),
            heat_conductivity: 0.6,
            ..Default::default()
        });
        
        // Wood
        self.register(MaterialDef {
            id: MaterialId::WOOD,
            name: "wood".to_string(),
            material_type: MaterialType::Solid,
            color: [139, 90, 43, 255],
            density: 0.6,
            hardness: Some(2),
            structural: true,
            flammable: true,
            ignition_temp: Some(300.0),
            burns_to: Some(MaterialId::AIR), // TODO: Ash material
            burn_rate: 0.02,
            ..Default::default()
        });
        
        // Fire
        self.register(MaterialDef {
            id: MaterialId::FIRE,
            name: "fire".to_string(),
            material_type: MaterialType::Gas,
            color: [255, 100, 0, 255],
            density: 0.0001,
            hardness: None,
            ..Default::default()
        });
        
        // Smoke
        self.register(MaterialDef {
            id: MaterialId::SMOKE,
            name: "smoke".to_string(),
            material_type: MaterialType::Gas,
            color: [60, 60, 60, 150],
            density: 0.001,
            hardness: None,
            ..Default::default()
        });
        
        // Steam
        self.register(MaterialDef {
            id: MaterialId::STEAM,
            name: "steam".to_string(),
            material_type: MaterialType::Gas,
            color: [200, 200, 200, 100],
            density: 0.0006,
            hardness: None,
            freezing_point: Some(100.0), // Condenses below boiling point
            freezes_to: Some(MaterialId::WATER),
            ..Default::default()
        });
        
        // Lava
        self.register(MaterialDef {
            id: MaterialId::LAVA,
            name: "lava".to_string(),
            material_type: MaterialType::Liquid,
            color: [255, 80, 0, 255],
            density: 3.0,
            hardness: None,
            viscosity: 0.8, // Very viscous
            freezing_point: Some(700.0),
            freezes_to: Some(MaterialId::STONE),
            heat_conductivity: 0.8,
            ..Default::default()
        });
        
        // Oil
        self.register(MaterialDef {
            id: MaterialId::OIL,
            name: "oil".to_string(),
            material_type: MaterialType::Liquid,
            color: [50, 40, 30, 255],
            density: 0.8, // Floats on water
            hardness: None,
            viscosity: 0.3,
            flammable: true,
            ignition_temp: Some(200.0),
            burns_to: Some(MaterialId::SMOKE),
            burn_rate: 0.05,
            ..Default::default()
        });
        
        // Acid
        self.register(MaterialDef {
            id: MaterialId::ACID,
            name: "acid".to_string(),
            material_type: MaterialType::Liquid,
            color: [0, 255, 0, 200],
            density: 1.1,
            hardness: None,
            viscosity: 0.2,
            ..Default::default()
        });
        
        // Ice
        self.register(MaterialDef {
            id: MaterialId::ICE,
            name: "ice".to_string(),
            material_type: MaterialType::Solid,
            color: [200, 230, 255, 200],
            density: 0.9,
            hardness: Some(2),
            structural: true,
            melting_point: Some(0.0),
            melts_to: Some(MaterialId::WATER),
            ..Default::default()
        });
        
        // Glass
        self.register(MaterialDef {
            id: MaterialId::GLASS,
            name: "glass".to_string(),
            material_type: MaterialType::Solid,
            color: [200, 220, 255, 150],
            density: 2.5,
            hardness: Some(3),
            structural: true,
            melting_point: Some(1400.0),
            melts_to: Some(MaterialId::LAVA), // Molten glass
            ..Default::default()
        });
        
        // Metal
        self.register(MaterialDef {
            id: MaterialId::METAL,
            name: "metal".to_string(),
            material_type: MaterialType::Solid,
            color: [180, 180, 190, 255],
            density: 7.8,
            hardness: Some(7),
            structural: true,
            melting_point: Some(1500.0),
            melts_to: Some(MaterialId::LAVA), // Molten metal
            heat_conductivity: 0.9,
            conducts_electricity: true,
            ..Default::default()
        });

        // Bedrock - indestructible foundation
        self.register(MaterialDef {
            id: MaterialId::BEDROCK,
            name: "bedrock".to_string(),
            material_type: MaterialType::Solid,
            color: [40, 40, 50, 255], // Dark gray
            density: 100.0,
            hardness: None, // None = indestructible
            structural: true,
            heat_conductivity: 0.1,
            ..Default::default()
        });
    }
    
    fn register(&mut self, material: MaterialDef) {
        let id = material.id as usize;
        
        // Ensure vec is large enough
        if self.materials.len() <= id {
            self.materials.resize(id + 1, MaterialDef::default());
        }
        
        self.materials[id] = material;
    }
    
    /// Get material definition by ID
    pub fn get(&self, id: u16) -> &MaterialDef {
        self.materials.get(id as usize).unwrap_or(&self.materials[0])
    }
    
    /// Get color for a material
    pub fn get_color(&self, id: u16) -> [u8; 4] {
        self.get(id).color
    }
}

impl Default for Materials {
    fn default() -> Self {
        Self::new()
    }
}
