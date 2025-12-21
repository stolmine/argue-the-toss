// Battlefield grid structure and management

use std::collections::HashMap;

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
}

/// Types of terrain on the battlefield
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Trench,
    NoMansLand,
    Mud,
    Fortification,
    Tree,
    CivilianBuilding,
}

impl TerrainType {
    /// Returns the movement cost multiplier for this terrain
    pub fn movement_cost(&self) -> f32 {
        match self {
            TerrainType::Trench => 1.0,
            TerrainType::NoMansLand => 1.5,
            TerrainType::Mud => 2.0,
            TerrainType::Fortification => 0.5,
            TerrainType::Tree => 1.8,
            TerrainType::CivilianBuilding => 0.8,
        }
    }

    /// Returns whether this terrain blocks line of sight
    pub fn blocks_los(&self) -> bool {
        matches!(
            self,
            TerrainType::Fortification | TerrainType::Tree | TerrainType::CivilianBuilding
        )
    }

    /// Returns the ASCII character representation
    pub fn to_char(&self) -> char {
        match self {
            TerrainType::Trench => '═',
            TerrainType::NoMansLand => '.',
            TerrainType::Mud => '~',
            TerrainType::Fortification => '▓',
            TerrainType::Tree => '♣',
            TerrainType::CivilianBuilding => '▓',
        }
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

/// The main battlefield grid structure
#[derive(Clone, Default)]
pub struct Battlefield {
    width: usize,
    height: usize,
    tiles: HashMap<Position, Tile>,
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
}
