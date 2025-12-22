// Turn state management for turn-based gameplay

use specs::Entity;
use std::collections::HashSet;

/// Global turn state resource
#[derive(Debug)]
pub struct TurnState {
    pub current_turn: u32,
    pub phase: TurnPhase,
    pub turn_order_mode: TurnOrderMode,
    pub entities_ready: HashSet<Entity>,
}

impl TurnState {
    pub fn new() -> Self {
        Self::new_with_mode(TurnOrderMode::PlayerFirst)
    }

    pub fn new_with_mode(turn_order_mode: TurnOrderMode) -> Self {
        Self {
            current_turn: 1,
            phase: TurnPhase::Planning,
            turn_order_mode,
            entities_ready: HashSet::new(),
        }
    }

    pub fn is_entity_ready(&self, entity: Entity) -> bool {
        self.entities_ready.contains(&entity)
    }

    pub fn mark_entity_ready(&mut self, entity: Entity) {
        self.entities_ready.insert(entity);
    }

    pub fn reset_for_new_turn(&mut self) {
        self.current_turn += 1;
        self.entities_ready.clear();
        self.phase = TurnPhase::Planning;
    }
}

impl Default for TurnState {
    fn default() -> Self {
        Self::new()
    }
}

/// Turn phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnPhase {
    /// Entities planning/committing actions
    Planning,
    /// All committed actions resolve
    Execution,
    /// Post-execution cleanup, damage application, etc.
    Resolution,
}

/// Turn order modes for experimentation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnOrderMode {
    /// Player-controlled entity acts first, then all NPCs
    PlayerFirst,
    /// All entities act, resolve together (future)
    Simultaneous,
    /// Speed stat determines action order (future)
    InitiativeBased,
}
