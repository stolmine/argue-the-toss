// Facing Component
// Tracks which direction an entity is facing (for vision cones, auto-facing, etc.)

use specs::{Component, VecStorage};

/// Eight cardinal and intercardinal directions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction8 {
    N,   // North (0°)
    NE,  // Northeast (45°)
    E,   // East (90°)
    SE,  // Southeast (135°)
    S,   // South (180°)
    SW,  // Southwest (225°)
    W,   // West (270°)
    NW,  // Northwest (315°)
}

impl Direction8 {
    /// Create direction from movement delta
    /// Returns None if dx and dy are both 0 (no movement)
    pub fn from_movement(dx: i32, dy: i32) -> Option<Self> {
        match (dx.signum(), dy.signum()) {
            (0, -1) => Some(Direction8::N),
            (1, -1) => Some(Direction8::NE),
            (1, 0) => Some(Direction8::E),
            (1, 1) => Some(Direction8::SE),
            (0, 1) => Some(Direction8::S),
            (-1, 1) => Some(Direction8::SW),
            (-1, 0) => Some(Direction8::W),
            (-1, -1) => Some(Direction8::NW),
            _ => None, // (0, 0) - no movement
        }
    }

    /// Rotate clockwise (90 degree increments)
    pub fn rotate_cw(&self) -> Self {
        match self {
            Direction8::N => Direction8::NE,
            Direction8::NE => Direction8::E,
            Direction8::E => Direction8::SE,
            Direction8::SE => Direction8::S,
            Direction8::S => Direction8::SW,
            Direction8::SW => Direction8::W,
            Direction8::W => Direction8::NW,
            Direction8::NW => Direction8::N,
        }
    }

    /// Rotate counter-clockwise (90 degree increments)
    pub fn rotate_ccw(&self) -> Self {
        match self {
            Direction8::N => Direction8::NW,
            Direction8::NW => Direction8::W,
            Direction8::W => Direction8::SW,
            Direction8::SW => Direction8::S,
            Direction8::S => Direction8::SE,
            Direction8::SE => Direction8::E,
            Direction8::E => Direction8::NE,
            Direction8::NE => Direction8::N,
        }
    }

    /// Get angle in degrees (0° = North, clockwise)
    pub fn angle_degrees(&self) -> f32 {
        match self {
            Direction8::N => 0.0,
            Direction8::NE => 45.0,
            Direction8::E => 90.0,
            Direction8::SE => 135.0,
            Direction8::S => 180.0,
            Direction8::SW => 225.0,
            Direction8::W => 270.0,
            Direction8::NW => 315.0,
        }
    }

    /// Get direction vector (dx, dy)
    pub fn to_vector(&self) -> (i32, i32) {
        match self {
            Direction8::N => (0, -1),
            Direction8::NE => (1, -1),
            Direction8::E => (1, 0),
            Direction8::SE => (1, 1),
            Direction8::S => (0, 1),
            Direction8::SW => (-1, 1),
            Direction8::W => (-1, 0),
            Direction8::NW => (-1, -1),
        }
    }

    /// Get display character for facing indicator
    pub fn to_char(&self) -> char {
        match self {
            Direction8::N => '↑',
            Direction8::NE => '↗',
            Direction8::E => '→',
            Direction8::SE => '↘',
            Direction8::S => '↓',
            Direction8::SW => '↙',
            Direction8::W => '←',
            Direction8::NW => '↖',
        }
    }
}

impl Default for Direction8 {
    fn default() -> Self {
        Direction8::N
    }
}

/// Component: Entity's current facing direction
#[derive(Debug, Clone)]
pub struct Facing {
    pub direction: Direction8,
}

impl Component for Facing {
    type Storage = VecStorage<Self>;
}

impl Facing {
    pub fn new(direction: Direction8) -> Self {
        Self { direction }
    }

    /// Update facing based on movement
    pub fn update_from_movement(&mut self, dx: i32, dy: i32) {
        if let Some(dir) = Direction8::from_movement(dx, dy) {
            self.direction = dir;
        }
    }

    /// Rotate facing clockwise
    pub fn rotate_cw(&mut self) {
        self.direction = self.direction.rotate_cw();
    }

    /// Rotate facing counter-clockwise
    pub fn rotate_ccw(&mut self) {
        self.direction = self.direction.rotate_ccw();
    }
}

impl Default for Facing {
    fn default() -> Self {
        Self {
            direction: Direction8::N,
        }
    }
}
