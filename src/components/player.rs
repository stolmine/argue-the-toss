// Player marker component

use specs::{Component, NullStorage};

/// Marker component indicating this entity is controlled by the player
#[derive(Debug, Default, Clone, Copy)]
pub struct Player;

impl Component for Player {
    type Storage = NullStorage<Self>;
}
