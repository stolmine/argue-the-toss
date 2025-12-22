// Battlefield Generation Configuration
// Defines parameters for procedural battlefield generation

/// Type of battlefield to generate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattlefieldType {
    /// Western Front style (trenches, mud, fortifications)
    WesternFront,
    /// Eastern Front style (more open, less trenches, villages)
    EasternFront,
    /// Urban combat (buildings, rubble, streets)
    Urban,
    /// Village/Town (mix of buildings and natural terrain)
    Village,
    /// Open field with natural features
    OpenField,
}

/// Density of trench networks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrenchDensity {
    None,
    Sparse,      // 10-20% coverage
    Moderate,    // 30-40% coverage
    Dense,       // 50-60% coverage
    VeryDense,   // 70%+ coverage
}

/// Level of fortification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FortificationLevel {
    None,
    Light,       // Basic sandbags
    Moderate,    // Sandbags + some bunkers
    Heavy,       // Extensive fortifications
    Fortress,    // Maximum fortifications
}

/// Comprehensive battlefield generation configuration
#[derive(Debug, Clone)]
pub struct BattlefieldGenerationConfig {
    // Map dimensions
    pub width: usize,
    pub height: usize,

    // Map type and style
    pub battlefield_type: BattlefieldType,
    pub trench_density: TrenchDensity,
    pub fortification_level: FortificationLevel,

    // Terrain features
    pub mud_coverage: f32,          // 0.0 to 1.0
    pub crater_density: f32,        // Craters per 100 tiles
    pub water_features: bool,       // Rivers, ponds, flooded craters
    pub forest_coverage: f32,       // 0.0 to 1.0
    pub building_density: f32,      // Buildings per 100 tiles

    // Tactical features
    pub barbed_wire_coverage: f32,  // 0.0 to 1.0 (in no-man's land)
    pub mg_nest_count: usize,       // Number of MG nests per side
    pub bunker_count: usize,        // Number of bunkers per side

    // Generation parameters
    pub seed: u64,                  // For reproducible generation
    pub no_mans_land_width: usize,  // Width of area between trench lines

    // Faction positions
    pub allies_side: Side,          // Which side allies spawn (North/South/East/West)
}

/// Which side of the map a faction occupies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    North,
    South,
    East,
    West,
}

impl Default for BattlefieldGenerationConfig {
    fn default() -> Self {
        Self {
            width: 100,
            height: 100,
            battlefield_type: BattlefieldType::WesternFront,
            trench_density: TrenchDensity::Moderate,
            fortification_level: FortificationLevel::Moderate,
            mud_coverage: 0.3,
            crater_density: 2.0,
            water_features: true,
            forest_coverage: 0.1,
            building_density: 0.5,
            barbed_wire_coverage: 0.4,
            mg_nest_count: 3,
            bunker_count: 2,
            seed: 12345,
            no_mans_land_width: 20,
            allies_side: Side::South,
        }
    }
}

impl BattlefieldGenerationConfig {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: Set map dimensions
    pub fn with_dimensions(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Builder: Set battlefield type
    pub fn with_type(mut self, battlefield_type: BattlefieldType) -> Self {
        self.battlefield_type = battlefield_type;
        self
    }

    /// Builder: Set trench density
    pub fn with_trench_density(mut self, density: TrenchDensity) -> Self {
        self.trench_density = density;
        self
    }

    /// Builder: Set fortification level
    pub fn with_fortifications(mut self, level: FortificationLevel) -> Self {
        self.fortification_level = level;
        self
    }

    /// Builder: Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Builder: Set allies spawn side
    pub fn with_allies_side(mut self, side: Side) -> Self {
        self.allies_side = side;
        self
    }

    /// Preset: Battle of Verdun (dense trenches, heavy fortifications, mud)
    pub fn verdun() -> Self {
        Self {
            battlefield_type: BattlefieldType::WesternFront,
            trench_density: TrenchDensity::VeryDense,
            fortification_level: FortificationLevel::Fortress,
            mud_coverage: 0.5,
            crater_density: 4.0,
            water_features: true,
            forest_coverage: 0.05,
            building_density: 0.2,
            barbed_wire_coverage: 0.6,
            mg_nest_count: 5,
            bunker_count: 4,
            no_mans_land_width: 15,
            ..Default::default()
        }
    }

    /// Preset: Battle of the Somme (moderate trenches, craters, wire)
    pub fn somme() -> Self {
        Self {
            battlefield_type: BattlefieldType::WesternFront,
            trench_density: TrenchDensity::Dense,
            fortification_level: FortificationLevel::Moderate,
            mud_coverage: 0.4,
            crater_density: 3.5,
            water_features: true,
            forest_coverage: 0.1,
            building_density: 0.3,
            barbed_wire_coverage: 0.5,
            mg_nest_count: 4,
            bunker_count: 3,
            no_mans_land_width: 25,
            ..Default::default()
        }
    }

    /// Preset: Battle of Ypres (flooded craters, moderate trenches)
    pub fn ypres() -> Self {
        Self {
            battlefield_type: BattlefieldType::WesternFront,
            trench_density: TrenchDensity::Moderate,
            fortification_level: FortificationLevel::Moderate,
            mud_coverage: 0.6,
            crater_density: 3.0,
            water_features: true,
            forest_coverage: 0.08,
            building_density: 0.4,
            barbed_wire_coverage: 0.45,
            mg_nest_count: 3,
            bunker_count: 2,
            no_mans_land_width: 20,
            ..Default::default()
        }
    }

    /// Preset: Eastern Front - Tannenberg (open terrain, less trenches, forests)
    pub fn tannenberg() -> Self {
        Self {
            battlefield_type: BattlefieldType::EasternFront,
            trench_density: TrenchDensity::Sparse,
            fortification_level: FortificationLevel::Light,
            mud_coverage: 0.2,
            crater_density: 1.0,
            water_features: true,
            forest_coverage: 0.3,
            building_density: 0.8,
            barbed_wire_coverage: 0.1,
            mg_nest_count: 2,
            bunker_count: 1,
            no_mans_land_width: 40,
            ..Default::default()
        }
    }

    /// Preset: Village combat (buildings, streets, light fortifications)
    pub fn village() -> Self {
        Self {
            battlefield_type: BattlefieldType::Village,
            trench_density: TrenchDensity::Sparse,
            fortification_level: FortificationLevel::Light,
            mud_coverage: 0.15,
            crater_density: 1.5,
            water_features: false,
            forest_coverage: 0.15,
            building_density: 3.0,
            barbed_wire_coverage: 0.2,
            mg_nest_count: 2,
            bunker_count: 1,
            no_mans_land_width: 30,
            ..Default::default()
        }
    }

    /// Preset: Urban combat (dense buildings, rubble, streets)
    pub fn urban() -> Self {
        Self {
            battlefield_type: BattlefieldType::Urban,
            trench_density: TrenchDensity::None,
            fortification_level: FortificationLevel::Moderate,
            mud_coverage: 0.05,
            crater_density: 2.0,
            water_features: false,
            forest_coverage: 0.0,
            building_density: 5.0,
            barbed_wire_coverage: 0.3,
            mg_nest_count: 4,
            bunker_count: 2,
            no_mans_land_width: 20,
            ..Default::default()
        }
    }

    /// Preset: Open field battle (minimal cover, natural terrain)
    pub fn open_field() -> Self {
        Self {
            battlefield_type: BattlefieldType::OpenField,
            trench_density: TrenchDensity::None,
            fortification_level: FortificationLevel::None,
            mud_coverage: 0.1,
            crater_density: 0.5,
            water_features: true,
            forest_coverage: 0.2,
            building_density: 0.1,
            barbed_wire_coverage: 0.0,
            mg_nest_count: 0,
            bunker_count: 0,
            no_mans_land_width: 50,
            ..Default::default()
        }
    }
}

impl TrenchDensity {
    /// Get the coverage percentage for this density level
    pub fn coverage_percentage(&self) -> f32 {
        match self {
            TrenchDensity::None => 0.0,
            TrenchDensity::Sparse => 0.15,
            TrenchDensity::Moderate => 0.35,
            TrenchDensity::Dense => 0.55,
            TrenchDensity::VeryDense => 0.75,
        }
    }
}

impl FortificationLevel {
    /// Get counts for this fortification level (returns: sandbags, bunkers, mg_nests per side)
    pub fn get_counts(&self) -> (usize, usize, usize) {
        match self {
            FortificationLevel::None => (0, 0, 0),
            FortificationLevel::Light => (5, 0, 1),
            FortificationLevel::Moderate => (10, 2, 3),
            FortificationLevel::Heavy => (20, 4, 5),
            FortificationLevel::Fortress => (30, 6, 7),
        }
    }
}
