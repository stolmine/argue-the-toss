// Game configuration settings

use crate::game_logic::turn_state::TurnOrderMode;

/// Global game configuration
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Time budget per turn in seconds (5.0-30.0 range)
    pub time_budget_seconds: f32,
    /// Turn order mode
    pub turn_order_mode: TurnOrderMode,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            time_budget_seconds: 10.0,
            turn_order_mode: TurnOrderMode::PlayerFirst,
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
