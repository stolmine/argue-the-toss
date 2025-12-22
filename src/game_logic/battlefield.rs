// Battlefield grid structure and management

use std::collections::HashMap;
use super::terrain_properties::TerrainProperties;

/// Represents a coordinate on the battlefield
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// Manhattan distance (for grid-based calculations)
    pub fn manhattan_distance_to(&self, other: &Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

/// Types of terrain on the battlefield
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    // Basic terrain
    NoMansLand,
    Grass,
    Mud,
    Water,
    DeepWater,

    // Multi-tile trench structures
    TrenchFloor,
    TrenchParapet,
    TrenchRamp,

    // Legacy trench (backward compatible)
    Trench,

    // Fortifications
    Sandbags,
    Bunker,
    MgNest,
    BarbedWire,

    // Natural obstacles
    Tree,
    Forest,
    Hedge,
    Rubble,

    // Craters
    ShellCrater,
    CraterWater,

    // Buildings
    BuildingWall,
    BuildingFloor,
    BuildingDoor,
    BuildingWindow,
    Ruins,

    // Roads
    Road,
    Path,

    // Communication trenches
    CommTrench,

    // Special (backward compatible)
    Fortification,
    CivilianBuilding,
}

impl TerrainType {
    /// Get the comprehensive properties for this terrain type
    pub fn properties(&self) -> TerrainProperties {
        match self {
            // Basic terrain
            TerrainType::NoMansLand => TerrainProperties::NO_MANS_LAND,
            TerrainType::Grass => TerrainProperties::GRASS,
            TerrainType::Mud => TerrainProperties::MUD,
            TerrainType::Water => TerrainProperties::WATER,
            TerrainType::DeepWater => TerrainProperties::DEEP_WATER,

            // Multi-tile trench structures
            TerrainType::TrenchFloor => TerrainProperties::TRENCH_FLOOR,
            TerrainType::TrenchParapet => TerrainProperties::TRENCH_PARAPET,
            TerrainType::TrenchRamp => TerrainProperties::TRENCH_RAMP,

            // Legacy trench
            TerrainType::Trench => TerrainProperties::TRENCH,

            // Fortifications
            TerrainType::Sandbags => TerrainProperties::SANDBAGS,
            TerrainType::Bunker => TerrainProperties::BUNKER,
            TerrainType::MgNest => TerrainProperties::MG_NEST,
            TerrainType::BarbedWire => TerrainProperties::BARBED_WIRE,

            // Natural obstacles
            TerrainType::Tree => TerrainProperties::TREE,
            TerrainType::Forest => TerrainProperties::FOREST,
            TerrainType::Hedge => TerrainProperties::HEDGE,
            TerrainType::Rubble => TerrainProperties::RUBBLE,

            // Craters
            TerrainType::ShellCrater => TerrainProperties::SHELL_CRATER,
            TerrainType::CraterWater => TerrainProperties::CRATER_WATER,

            // Buildings
            TerrainType::BuildingWall => TerrainProperties::BUILDING_WALL,
            TerrainType::BuildingFloor => TerrainProperties::BUILDING_FLOOR,
            TerrainType::BuildingDoor => TerrainProperties::BUILDING_DOOR,
            TerrainType::BuildingWindow => TerrainProperties::BUILDING_WINDOW,
            TerrainType::Ruins => TerrainProperties::RUINS,

            // Roads
            TerrainType::Road => TerrainProperties::ROAD,
            TerrainType::Path => TerrainProperties::PATH,

            // Communication trenches
            TerrainType::CommTrench => TerrainProperties::COMM_TRENCH,

            // Special (backward compatible)
            TerrainType::Fortification => TerrainProperties::FORTIFICATION,
            TerrainType::CivilianBuilding => TerrainProperties::CIVILIAN_BUILDING,
        }
    }

    /// Returns the movement cost multiplier for this terrain (backward compatible)
    pub fn movement_cost(&self) -> f32 {
        self.properties().movement_cost
    }

    /// Returns whether this terrain blocks line of sight (backward compatible)
    pub fn blocks_los(&self) -> bool {
        self.properties().blocks_los()
    }

    /// Returns the ASCII character representation (backward compatible)
    pub fn to_char(&self) -> char {
        self.properties().character
    }

    /// Returns whether this terrain is passable
    pub fn is_passable(&self) -> bool {
        self.properties().is_passable
    }

    /// Returns the cover bonus for combat
    pub fn cover_bonus(&self) -> f32 {
        self.properties().cover_bonus
    }
}

/// Represents a tile on the battlefield
#[derive(Debug, Clone)]
pub struct Tile {
    pub terrain: TerrainType,
    pub explored: bool,
    pub visible: bool,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            terrain: TerrainType::NoMansLand,
            explored: false,
            visible: false,
        }
    }
}

/// Spawn zone for a faction
#[derive(Debug, Clone)]
pub struct SpawnZone {
    pub center: Position,
    pub radius: usize,
}

impl SpawnZone {
    pub fn new(center: Position, radius: usize) -> Self {
        Self { center, radius }
    }

    pub fn contains(&self, pos: &Position) -> bool {
        self.center.distance_to(pos) <= self.radius as f32
    }
}

/// The main battlefield grid structure
#[derive(Clone)]
pub struct Battlefield {
    width: usize,
    height: usize,
    tiles: HashMap<Position, Tile>,
    pub ally_spawn: Option<SpawnZone>,
    pub enemy_spawn: Option<SpawnZone>,
}

impl Default for Battlefield {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            tiles: HashMap::new(),
            ally_spawn: None,
            enemy_spawn: None,
        }
    }
}

impl Battlefield {
    /// Creates a new battlefield with the given dimensions
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = HashMap::new();

        // Initialize all tiles with default terrain
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                tiles.insert(Position::new(x, y), Tile::default());
            }
        }

        Self {
            width,
            height,
            tiles,
            ally_spawn: None,
            enemy_spawn: None,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Gets a tile at the given position
    pub fn get_tile(&self, pos: &Position) -> Option<&Tile> {
        self.tiles.get(pos)
    }

    /// Gets a mutable tile at the given position
    pub fn get_tile_mut(&mut self, pos: &Position) -> Option<&mut Tile> {
        self.tiles.get_mut(pos)
    }

    /// Sets terrain type at a position
    pub fn set_terrain(&mut self, pos: Position, terrain: TerrainType) {
        if let Some(tile) = self.tiles.get_mut(&pos) {
            tile.terrain = terrain;
        }
    }

    /// Checks if a position is within battlefield bounds
    pub fn in_bounds(&self, pos: &Position) -> bool {
        pos.x >= 0 && pos.x < self.width as i32 && pos.y >= 0 && pos.y < self.height as i32
    }

    /// Marks a tile as visible (for fog of war)
    pub fn set_visible(&mut self, pos: Position, visible: bool) {
        if let Some(tile) = self.tiles.get_mut(&pos) {
            tile.visible = visible;
            if visible {
                tile.explored = true;
            }
        }
    }

    /// Resets all tiles to not visible (call at start of each visibility update)
    pub fn reset_visibility(&mut self) {
        for tile in self.tiles.values_mut() {
            tile.visible = false;
        }
    }

    /// Set the spawn zones for both factions
    pub fn set_spawn_zones(&mut self, ally_spawn: SpawnZone, enemy_spawn: SpawnZone) {
        self.ally_spawn = Some(ally_spawn);
        self.enemy_spawn = Some(enemy_spawn);
    }

    /// Get spawn positions for a faction
    pub fn get_spawn_positions(&self, is_allies: bool, count: usize) -> Vec<Position> {
        use rand::Rng;

        let spawn_zone = if is_allies {
            &self.ally_spawn
        } else {
            &self.enemy_spawn
        };

        let zone = match spawn_zone {
            Some(z) => z,
            None => return vec![],
        };

        let mut rng = rand::thread_rng();
        let mut positions = Vec::new();
        let mut attempts = 0;
        let max_attempts = count * 50;

        while positions.len() < count && attempts < max_attempts {
            attempts += 1;

            let offset_x = rng.random_range(-(zone.radius as i32)..=(zone.radius as i32));
            let offset_y = rng.random_range(-(zone.radius as i32)..=(zone.radius as i32));

            let pos = Position::new(zone.center.x + offset_x, zone.center.y + offset_y);

            if !self.in_bounds(&pos) {
                continue;
            }

            if !zone.contains(&pos) {
                continue;
            }

            if positions.iter().any(|p: &Position| p.distance_to(&pos) < 2.0) {
                continue;
            }

            if let Some(tile) = self.get_tile(&pos) {
                let terrain = tile.terrain;
                if !terrain.is_passable() || matches!(terrain, TerrainType::Water | TerrainType::DeepWater) {
                    continue;
                }
            }

            positions.push(pos);
        }

        positions
    }
}
