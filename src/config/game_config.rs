// Game configuration settings

use crate::game_logic::turn_state::TurnOrderMode;

/// Global game configuration
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Time budget per turn in seconds (5.0-30.0 range)
    pub time_budget_seconds: f32,
    /// Turn order mode
    pub turn_order_mode: TurnOrderMode,
    /// Movement time cost per tile (seconds)
    pub movement_time_cost: f32,
    /// Rotation time cost (seconds)
    pub rotation_time_cost: f32,
    /// Tile scale in meters (for UI display)
    pub tile_scale_meters: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            time_budget_seconds: 12.0,  // Updated: 10.0 -> 12.0
            turn_order_mode: TurnOrderMode::PlayerFirst,
            movement_time_cost: 1.5,     // New: 1.5s per tile
            rotation_time_cost: 0.3,     // New: 0.3s per rotation
            tile_scale_meters: 2.0,      // New: ~2 meters per tile
        }
    }
}

impl GameConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set time budget with validation (5-30 seconds)
    pub fn with_time_budget(mut self, seconds: f32) -> Self {
        self.time_budget_seconds = seconds.clamp(5.0, 30.0);
        self
    }

    /// Set turn order mode
    pub fn with_turn_order_mode(mut self, mode: TurnOrderMode) -> Self {
        self.turn_order_mode = mode;
        self
    }
}
