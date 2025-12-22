// Action component for turn-based action system

use specs::{Component, Entity, VecStorage};

/// Types of actions entities can perform
#[derive(Debug, Clone)]
pub enum ActionType {
    /// Move in a direction with terrain cost multiplier
    Move { dx: i32, dy: i32, terrain_cost: f32 },
    /// Rotate facing direction (true = clockwise, false = counter-clockwise)
    Rotate { clockwise: bool },
    /// Shoot at a target entity
    Shoot { target: Entity },
    /// Reload weapon
    Reload,
    /// Throw grenade at position
    ThrowGrenade { target_x: i32, target_y: i32 },
    /// Wait/do nothing
    Wait,
}

impl ActionType {
    /// Get the base time cost for this action type (in seconds)
    /// Note: Movement and rotation costs should be read from GameConfig in the future
    pub fn base_time_cost(&self) -> f32 {
        match self {
            ActionType::Move { terrain_cost, .. } => 1.5 * terrain_cost, // Updated: 2.0 -> 1.5
            ActionType::Rotate { .. } => 0.3, // New: Rotation cost
            ActionType::Shoot { .. } => 3.0,
            ActionType::Reload => 5.0,
            ActionType::ThrowGrenade { .. } => 4.0,
            ActionType::Wait => 1.0,
        }
    }
}

/// Component: A queued action on an entity
#[derive(Debug, Clone)]
pub struct QueuedAction {
    pub action_type: ActionType,
    pub time_cost: f32,
    pub committed: bool,
}

impl Component for QueuedAction {
    type Storage = VecStorage<Self>;
}

impl QueuedAction {
    pub fn new(action_type: ActionType) -> Self {
        let time_cost = action_type.base_time_cost();
        Self {
            action_type,
            time_cost,
            committed: true,
        }
    }
}

/// Component: An ongoing multi-turn action
#[derive(Debug, Clone)]
pub struct OngoingAction {
    pub action_type: ActionType,
    pub total_time: f32,
    pub time_completed: f32,
    pub locked: bool, // Cannot cancel
}

impl Component for OngoingAction {
    type Storage = VecStorage<Self>;
}

impl OngoingAction {
    pub fn new(action_type: ActionType) -> Self {
        let total_time = action_type.base_time_cost();
        Self {
            action_type,
            total_time,
            time_completed: 0.0,
            locked: true,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.time_completed >= self.total_time
    }

    pub fn progress_time(&mut self, delta: f32) {
        self.time_completed += delta;
    }
}
