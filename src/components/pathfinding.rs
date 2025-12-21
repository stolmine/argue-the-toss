// Pathfinding component for multi-step movement paths

use crate::game_logic::battlefield::{Battlefield, Position};
use specs::{Component, VecStorage};

/// A planned multi-step path for an entity
#[derive(Debug, Clone)]
pub struct PlannedPath {
    /// Remaining steps in the path (next step is at index 0)
    pub steps: Vec<Position>,
    /// Total estimated time cost for remaining path
    pub total_cost: f32,
    /// Whether this path should be rendered (for visual preview)
    pub show_preview: bool,
}

impl Component for PlannedPath {
    type Storage = VecStorage<Self>;
}

impl PlannedPath {
    /// Create a new planned path
    pub fn new(steps: Vec<Position>, total_cost: f32, show_preview: bool) -> Self {
        Self {
            steps,
            total_cost,
            show_preview,
        }
    }

    /// Get the next step and remove it from path
    pub fn pop_next(&mut self) -> Option<Position> {
        if !self.steps.is_empty() {
            Some(self.steps.remove(0))
        } else {
            None
        }
    }

    /// Check if path is complete (no more steps)
    pub fn is_complete(&self) -> bool {
        self.steps.is_empty()
    }

    /// Validate if path is still traversable
    /// Returns true if path is valid, false if it should be invalidated
    pub fn is_valid(&self, _current_pos: &crate::components::position::Position, _battlefield: &Battlefield) -> bool {
        // Future: implement collision detection and obstacle checking
        // For now, assume paths remain valid
        true
    }
}
