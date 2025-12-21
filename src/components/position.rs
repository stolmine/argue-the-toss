// Position component for entity placement on battlefield

use crate::game_logic::battlefield::Position as BattlefieldPosition;
use specs::{Component, VecStorage};

/// Component that represents an entity's position on the battlefield
#[derive(Debug, Clone, Copy)]
pub struct Position(pub BattlefieldPosition);

impl Component for Position {
    type Storage = VecStorage<Self>;
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self(BattlefieldPosition::new(x, y))
    }

    pub fn x(&self) -> i32 {
        self.0.x
    }

    pub fn y(&self) -> i32 {
        self.0.y
    }

    pub fn as_battlefield_pos(&self) -> &BattlefieldPosition {
        &self.0
    }
}
