// Path Execution System
// Converts PlannedPath components into individual Move actions step-by-step

use crate::components::{
    action::{ActionType, QueuedAction},
    pathfinding::PlannedPath,
    position::Position,
    time_budget::TimeBudget,
};
use crate::game_logic::{battlefield::Battlefield, turn_state::{TurnPhase, TurnState}};
use crate::utils::event_log::EventLog;
use specs::{Entities, Join, Read, ReadStorage, System, Write, WriteStorage};

pub struct PathExecutionSystem;

impl<'a> System<'a> for PathExecutionSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, PlannedPath>,
        WriteStorage<'a, QueuedAction>,
        WriteStorage<'a, TimeBudget>,
        Read<'a, Battlefield>,
        Read<'a, TurnState>,
        Write<'a, EventLog>,
    );

    fn run(
        &mut self,
        (entities, positions, mut paths, mut queued, mut budgets, battlefield, turn_state, mut _log): Self::SystemData,
    ) {
        // Only execute during Planning phase (before actions are executed)
        if !matches!(turn_state.phase, TurnPhase::Planning) {
            return;
        }

        // Track which paths to remove (completed or invalid)
        let mut paths_to_remove = Vec::new();

        for (entity, pos, path) in (&entities, &positions, &mut paths).join() {
            // Skip if entity already has a queued action for this turn
            if queued.contains(entity) {
                continue;
            }

            // Skip if entity has no time budget (shouldn't happen, but safety check)
            let budget = match budgets.get_mut(entity) {
                Some(b) => b,
                None => continue,
            };

            // Skip if entity is out of time
            if budget.available_time() <= 0.0 {
                continue;
            }

            // Validate path is still traversable
            if !path.is_valid(pos, &battlefield) {
                // Path invalidated - mark for removal
                paths_to_remove.push(entity);
                continue;
            }

            // Get next step from path (battlefield::Position)
            if let Some(next_pos) = path.pop_next() {
                // Calculate delta from current position
                let dx = next_pos.x - pos.x();
                let dy = next_pos.y - pos.y();

                // Get terrain cost for the destination tile
                let terrain_cost = battlefield
                    .get_tile(&next_pos)
                    .map(|t| t.terrain.movement_cost())
                    .unwrap_or(1.0);

                // Create Move action for this single step
                let action = ActionType::Move {
                    dx,
                    dy,
                    terrain_cost,
                };

                // Consume time budget for the action
                let time_cost = action.base_time_cost();
                budget.consume_time(time_cost);

                // Queue the action (will be processed by ActionExecutionSystem)
                queued
                    .insert(entity, QueuedAction::new(action))
                    .ok();
            }

            // If path is now complete, mark for removal
            if path.is_complete() {
                paths_to_remove.push(entity);
            }
        }

        // Clean up completed or invalid paths
        for entity in paths_to_remove {
            paths.remove(entity);
        }
    }
}
