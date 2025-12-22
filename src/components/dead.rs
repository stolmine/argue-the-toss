// Dead marker component

use specs::{Component, NullStorage};

/// Marker component: Entity is dead and should not act
#[derive(Debug, Clone, Copy, Default)]
pub struct Dead;

impl Component for Dead {
    type Storage = NullStorage<Self>;
}
