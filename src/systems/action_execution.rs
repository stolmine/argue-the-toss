// Action Execution System
// Executes committed actions during Execution phase

use crate::components::{
    action::{ActionType, OngoingAction, QueuedAction},
    position::Position,
};
use crate::game_logic::turn_state::{TurnPhase, TurnState};
use crate::utils::event_log::EventLog;
use specs::{Entities, Join, Read, ReadStorage, System, Write, WriteStorage};

pub struct ActionExecutionSystem;

impl<'a> System<'a> for ActionExecutionSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, QueuedAction>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, OngoingAction>,
        Write<'a, EventLog>,
        Read<'a, TurnState>,
    );

    fn run(
        &mut self,
        (entities, queued, mut positions, mut _ongoing, mut log, turn_state): Self::SystemData,
    ) {
        // Only execute during Execution phase
        if !matches!(turn_state.phase, TurnPhase::Execution) {
            return;
        }

        // Execute ALL committed actions (player, allies, enemies)
        for (_entity, action, pos) in (&entities, &queued, &mut positions).join() {
            if !action.committed {
                continue;
            }

            match &action.action_type {
                ActionType::Move {
                    dx,
                    dy,
                    terrain_cost: _,
                } => {
                    let new_x = pos.x() + dx;
                    let new_y = pos.y() + dy;
                    // Boundary check (from battlefield size)
                    if new_x >= 0 && new_x < 100 && new_y >= 0 && new_y < 100 {
                        *pos = Position::new(new_x, new_y);
                        log.add(format!("Entity moved to ({}, {})", new_x, new_y));
                    }
                }
                ActionType::Wait => {
                    // Waiting is a no-op execution
                }
                ActionType::Shoot { target: _ } => {
                    // Placeholder for future combat system
                    log.add("Entity shoots!".to_string());
                }
                ActionType::Reload => {
                    // Placeholder for future reload system
                    log.add("Entity reloads.".to_string());
                }
                ActionType::ThrowGrenade {
                    target_x: _,
                    target_y: _,
                } => {
                    // Placeholder for future grenade system
                    log.add("Entity throws grenade!".to_string());
                }
            }
        }

        // Clear executed actions after processing
        // (done by cleanup system in turn manager)
    }
}
