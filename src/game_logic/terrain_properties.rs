// Terrain Properties System
// Defines properties for all terrain types in a centralized, maintainable way

use ratatui::style::Color;

/// How terrain blocks line of sight
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LOSBlocking {
    /// Does not block LOS at all
    None,
    /// Partially blocks LOS (reduces visibility range or accuracy)
    Partial,
    /// Completely blocks LOS
    Full,
}

/// Comprehensive properties for a terrain type
#[derive(Debug, Clone)]
pub struct TerrainProperties {
    /// Display character for this terrain
    pub character: char,
    /// Display color
    pub color: Color,
    /// Movement cost multiplier (1.0 = normal, 2.0 = twice as slow)
    pub movement_cost: f32,
    /// Whether this terrain can be moved through
    pub is_passable: bool,
    /// How this terrain blocks line of sight
    pub los_blocking: LOSBlocking,
    /// Cover bonus for entities on this tile (0.0 = no cover, 0.9 = 90% cover)
    pub cover_bonus: f32,
    /// Human-readable name
    pub name: &'static str,
}

impl TerrainProperties {
    /// Check if terrain blocks LOS completely
    pub fn blocks_los(&self) -> bool {
        matches!(self.los_blocking, LOSBlocking::Full)
    }

    /// Check if terrain provides partial LOS blocking
    pub fn partially_blocks_los(&self) -> bool {
        matches!(self.los_blocking, LOSBlocking::Partial)
    }

    /// Get effective cover for damage calculation
    pub fn effective_cover(&self) -> f32 {
        self.cover_bonus.clamp(0.0, 0.95) // Maximum 95% damage reduction
    }
}

// Terrain property constants for easy reference
impl TerrainProperties {
    // Basic terrain types
    pub const NO_MANS_LAND: Self = Self {
        character: '.',
        color: Color::DarkGray,
        movement_cost: 1.5,
        is_passable: true,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.0,
        name: "No Man's Land",
    };

    pub const GRASS: Self = Self {
        character: ',',
        color: Color::Green,
        movement_cost: 1.0,
        is_passable: true,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.0,
        name: "Grass",
    };

    pub const MUD: Self = Self {
        character: '~',
        color: Color::Rgb(101, 67, 33), // Brown mud
        movement_cost: 2.5,
        is_passable: true,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.0,
        name: "Mud",
    };

    pub const WATER: Self = Self {
        character: '≈',
        color: Color::Blue,
        movement_cost: 5.0,
        is_passable: true, // Can wade through but very slow
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.1,
        name: "Water",
    };

    pub const DEEP_WATER: Self = Self {
        character: '≋',
        color: Color::Cyan,
        movement_cost: 100.0, // Effectively impassable
        is_passable: false,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.0,
        name: "Deep Water",
    };

    // Multi-tile trench structures
    pub const TRENCH_FLOOR: Self = Self {
        character: '▬',
        color: Color::Rgb(139, 90, 43), // Earth brown
        movement_cost: 1.0,
        is_passable: true,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.9, // Safest position
        name: "Trench Floor",
    };

    pub const TRENCH_PARAPET: Self = Self {
        character: '═',
        color: Color::Rgb(120, 80, 40), // Slightly darker brown
        movement_cost: 1.2,
        is_passable: true,
        los_blocking: LOSBlocking::Partial, // Blocks LOS for soldiers in floor
        cover_bonus: 0.6, // Moderate cover but exposed when shooting
        name: "Fire Step",
    };

    pub const TRENCH_RAMP: Self = Self {
        character: '/',
        color: Color::Rgb(130, 85, 42),
        movement_cost: 1.5,
        is_passable: true,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.4,
        name: "Trench Ramp",
    };

    // Legacy unified trench (for backward compatibility)
    pub const TRENCH: Self = Self {
        character: '═',
        color: Color::Rgb(139, 90, 43),
        movement_cost: 1.0,
        is_passable: true,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.7,
        name: "Trench",
    };

    // Fortifications
    pub const SANDBAGS: Self = Self {
        character: '▓',
        color: Color::Rgb(194, 178, 128), // Tan
        movement_cost: 1.8,
        is_passable: true,
        los_blocking: LOSBlocking::Partial,
        cover_bonus: 0.7,
        name: "Sandbags",
    };

    pub const BUNKER: Self = Self {
        character: '▓',
        color: Color::Gray,
        movement_cost: 0.5,
        is_passable: true,
        los_blocking: LOSBlocking::Full,
        cover_bonus: 0.95, // Excellent cover
        name: "Bunker",
    };

    pub const MG_NEST: Self = Self {
        character: '≡',
        color: Color::DarkGray,
        movement_cost: 1.0,
        is_passable: true,
        los_blocking: LOSBlocking::Partial,
        cover_bonus: 0.8,
        name: "MG Nest",
    };

    pub const BARBED_WIRE: Self = Self {
        character: '#',
        color: Color::Gray,
        movement_cost: 8.0, // Very slow to cross
        is_passable: true,
        los_blocking: LOSBlocking::Partial,
        cover_bonus: 0.0, // No protection, just obstacle
        name: "Barbed Wire",
    };

    // Natural obstacles
    pub const TREE: Self = Self {
        character: '♣',
        color: Color::Green,
        movement_cost: 1.8,
        is_passable: true,
        los_blocking: LOSBlocking::Full,
        cover_bonus: 0.3,
        name: "Tree",
    };

    pub const FOREST: Self = Self {
        character: '♠',
        color: Color::Rgb(34, 139, 34), // Forest green
        movement_cost: 2.0,
        is_passable: true,
        los_blocking: LOSBlocking::Full,
        cover_bonus: 0.4,
        name: "Forest",
    };

    pub const HEDGE: Self = Self {
        character: '‡',
        color: Color::Rgb(85, 107, 47), // Dark olive green
        movement_cost: 2.5,
        is_passable: true,
        los_blocking: LOSBlocking::Partial,
        cover_bonus: 0.5,
        name: "Hedge",
    };

    pub const RUBBLE: Self = Self {
        character: '▒',
        color: Color::Gray,
        movement_cost: 2.2,
        is_passable: true,
        los_blocking: LOSBlocking::Partial,
        cover_bonus: 0.5,
        name: "Rubble",
    };

    // Craters
    pub const SHELL_CRATER: Self = Self {
        character: 'O',
        color: Color::Rgb(101, 67, 33),
        movement_cost: 2.0,
        is_passable: true,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.6, // Good cover when inside
        name: "Shell Crater",
    };

    pub const CRATER_WATER: Self = Self {
        character: '◯',
        color: Color::Rgb(70, 130, 180), // Steel blue
        movement_cost: 3.5,
        is_passable: true,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.4,
        name: "Water-Filled Crater",
    };

    // Buildings
    pub const BUILDING_WALL: Self = Self {
        character: '█',
        color: Color::Rgb(139, 69, 19), // Saddle brown
        movement_cost: 100.0,
        is_passable: false,
        los_blocking: LOSBlocking::Full,
        cover_bonus: 0.0,
        name: "Building Wall",
    };

    pub const BUILDING_FLOOR: Self = Self {
        character: '·',
        color: Color::Rgb(210, 180, 140), // Tan
        movement_cost: 0.8,
        is_passable: true,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.2,
        name: "Building Interior",
    };

    pub const BUILDING_DOOR: Self = Self {
        character: '+',
        color: Color::Rgb(139, 69, 19),
        movement_cost: 1.2,
        is_passable: true,
        los_blocking: LOSBlocking::Partial,
        cover_bonus: 0.3,
        name: "Doorway",
    };

    pub const BUILDING_WINDOW: Self = Self {
        character: '▯',
        color: Color::Cyan,
        movement_cost: 100.0,
        is_passable: false,
        los_blocking: LOSBlocking::Partial, // Can see through but can't pass
        cover_bonus: 0.0,
        name: "Window",
    };

    pub const RUINS: Self = Self {
        character: '▓',
        color: Color::DarkGray,
        movement_cost: 2.0,
        is_passable: true,
        los_blocking: LOSBlocking::Partial,
        cover_bonus: 0.6,
        name: "Ruins",
    };

    // Roads and paths
    pub const ROAD: Self = Self {
        character: '=',
        color: Color::Rgb(169, 169, 169), // Dark gray
        movement_cost: 0.7,
        is_passable: true,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.0,
        name: "Road",
    };

    pub const PATH: Self = Self {
        character: '-',
        color: Color::Rgb(184, 134, 11), // Dark goldenrod
        movement_cost: 0.9,
        is_passable: true,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.0,
        name: "Path",
    };

    // Communication trenches
    pub const COMM_TRENCH: Self = Self {
        character: '║',
        color: Color::Rgb(120, 80, 40),
        movement_cost: 1.2,
        is_passable: true,
        los_blocking: LOSBlocking::None,
        cover_bonus: 0.8,
        name: "Communication Trench",
    };

    // Special terrain
    pub const FORTIFICATION: Self = Self {
        character: '▓',
        color: Color::Gray,
        movement_cost: 0.5,
        is_passable: true,
        los_blocking: LOSBlocking::Full,
        cover_bonus: 0.85,
        name: "Fortification",
    };

    pub const CIVILIAN_BUILDING: Self = Self {
        character: '▓',
        color: Color::Rgb(139, 69, 19),
        movement_cost: 0.8,
        is_passable: true,
        los_blocking: LOSBlocking::Full,
        cover_bonus: 0.7,
        name: "Civilian Building",
    };
}
