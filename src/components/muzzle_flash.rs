// Muzzle flash visual effect component
// Temporary visual indicator when a unit fires their weapon

use crate::components::position::Position;
use specs::{Component, VecStorage};

/// Muzzle flash effect - rendered for one frame then removed
#[derive(Debug, Clone)]
pub struct MuzzleFlash {
    pub position: Position,
}

impl Component for MuzzleFlash {
    type Storage = VecStorage<Self>;
}

impl MuzzleFlash {
    pub fn new(position: Position) -> Self {
        Self { position }
    }
}
