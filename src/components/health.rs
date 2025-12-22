// Health component for damage tracking

use specs::{Component, VecStorage};

/// Component: Entity health and damage state
#[derive(Debug, Clone)]
pub struct Health {
    pub current: i32,
    pub maximum: i32,
}

impl Component for Health {
    type Storage = VecStorage<Self>;
}

impl Health {
    /// Create a new health component with given max HP
    pub fn new(maximum: i32) -> Self {
        Self {
            current: maximum,
            maximum,
        }
    }

    /// Standard soldier health (100 HP)
    pub fn soldier() -> Self {
        Self::new(100)
    }

    /// Take damage, returns true if still alive
    pub fn take_damage(&mut self, damage: i32) -> bool {
        self.current = (self.current - damage).max(0);
        self.is_alive()
    }

    /// Heal damage (up to maximum)
    pub fn heal(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.maximum);
    }

    /// Check if entity is alive
    pub fn is_alive(&self) -> bool {
        self.current > 0
    }

    /// Check if entity is dead
    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }

    /// Get health percentage (0.0 - 1.0)
    pub fn percentage(&self) -> f32 {
        if self.maximum == 0 {
            0.0
        } else {
            self.current as f32 / self.maximum as f32
        }
    }

    /// Get health percentage as display value (0-100)
    pub fn percentage_display(&self) -> i32 {
        (self.percentage() * 100.0) as i32
    }
}
