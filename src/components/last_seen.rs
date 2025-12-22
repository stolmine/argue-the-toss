// Last Seen Marker component
// Tracks the last known position of entities that have left vision

use crate::components::position::Position;
use crate::components::soldier::{Faction, Rank};
use specs::{Component, VecStorage, Entity};

/// Marks the last known position of an entity that has left vision
/// These are "ghost" markers showing where enemies were last spotted
#[derive(Debug, Clone)]
pub struct LastSeenMarker {
    /// Last known position of the entity
    pub position: Position,
    /// Faction of the entity (for display)
    pub faction: Faction,
    /// Rank of the entity (for display)
    pub rank: Rank,
    /// How many turns ago was this entity last seen
    pub turns_ago: u32,
    /// The turn number when this entity was last seen
    pub last_seen_turn: u32,
    /// The original entity this marker tracks (if it still exists)
    pub tracked_entity: Entity,
}

impl Component for LastSeenMarker {
    type Storage = VecStorage<Self>;
}

impl LastSeenMarker {
    pub fn new(
        position: Position,
        faction: Faction,
        rank: Rank,
        current_turn: u32,
        tracked_entity: Entity,
    ) -> Self {
        Self {
            position,
            faction,
            rank,
            turns_ago: 0,
            last_seen_turn: current_turn,
            tracked_entity,
        }
    }

    /// Update the marker for a new turn
    pub fn update_turn(&mut self, current_turn: u32) {
        self.turns_ago = current_turn.saturating_sub(self.last_seen_turn);
    }

    /// Check if marker should expire (configurable timeout)
    pub fn should_expire(&self, max_turns: u32) -> bool {
        self.turns_ago >= max_turns
    }
}
