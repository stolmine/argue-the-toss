// Position Validation System
// Validates that positions are being updated correctly during action execution
//
// This system runs AFTER ActionExecutionSystem to detect if the "movement bug"
// has been reintroduced. It checks if movement actions were executed but positions
// didn't change, which indicates a system ordering or phase transition issue.
//
// In development/debug builds, this emits warnings to stderr.
// In release builds, this is a no-op for performance.

use crate::components::{
    action::QueuedAction,
    position::Position,
};
use crate::game_logic::turn_state::{TurnPhase, TurnState};
use specs::{Join, Read, ReadStorage, System};
use std::collections::HashMap;

pub struct PositionValidationSystem {
    // Track positions before execution phase
    last_positions: HashMap<specs::Entity, (i32, i32)>,
    validation_enabled: bool,
}

impl PositionValidationSystem {
    pub fn new() -> Self {
        Self {
            last_positions: HashMap::new(),
            validation_enabled: cfg!(debug_assertions), // Only enable in debug builds
        }
    }
}

impl<'a> System<'a> for PositionValidationSystem {
    type SystemData = (
        specs::Entities<'a>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, QueuedAction>,
        Read<'a, TurnState>,
    );

    fn run(&mut self, (entities, positions, _actions, turn_state): Self::SystemData) {
        if !self.validation_enabled {
            return;
        }

        match turn_state.phase {
            TurnPhase::Planning => {
                // At start of planning, snapshot current positions
                self.last_positions.clear();
                for (entity, pos) in (&entities, &positions).join() {
                    self.last_positions.insert(entity, (pos.x(), pos.y()));
                }
            }
            TurnPhase::Execution => {
                // During execution, positions should be changing
                // We don't validate here because execution might take multiple frames
            }
            TurnPhase::Resolution => {
                // After resolution, check if any positions changed during this turn
                // If event log showed movement but positions didn't change, warn
                let mut any_changes = false;
                for (entity, pos) in (&entities, &positions).join() {
                    if let Some(&(old_x, old_y)) = self.last_positions.get(&entity) {
                        if old_x != pos.x() || old_y != pos.y() {
                            any_changes = true;
                            break;
                        }
                    }
                }

                // This is a simple heuristic: if we're in Resolution phase, we had
                // actions (otherwise we wouldn't have left Planning), but no positions
                // changed, something might be wrong.
                //
                // Note: This can have false positives (e.g., all actions were Rotate/Wait)
                // but it's better to warn and investigate than to miss the bug.
                if !any_changes && !self.last_positions.is_empty() {
                    // Check if we actually had move actions
                    // (we can't do this easily without tracking more state,
                    // so we'll just emit a debug note)
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "PositionValidationSystem: WARNING - Turn completed but no positions changed. \
                        This could indicate the movement bug if move actions were queued."
                    );
                }
            }
        }
    }
}
