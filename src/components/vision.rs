// Vision component for entity sight capability

use specs::{Component, VecStorage};

/// Component: Entity vision capability
#[derive(Debug, Clone)]
pub struct Vision {
    pub range: i32,  // How far the entity can see in tiles
}

impl Component for Vision {
    type Storage = VecStorage<Self>;
}

impl Vision {
    pub fn new(range: i32) -> Self {
        Self { range }
    }
}

impl Default for Vision {
    fn default() -> Self {
        Self { range: 10 }  // Default 10 tile range
    }
}
