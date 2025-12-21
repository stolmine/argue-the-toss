// AI Action Planning System
// Generates actions for NPC entities

use crate::components::{
    action::{ActionType, QueuedAction},
    pathfinding::PlannedPath,
    player::Player,
    position::Position,
    soldier::Soldier,
    time_budget::TimeBudget,
};
use crate::game_logic::{
    battlefield::Battlefield,
    pathfinding::calculate_path,
    turn_state::{TurnOrderMode, TurnPhase, TurnState},
};
use specs::{Entities, Join, Read, ReadStorage, System, WriteStorage};

pub struct AIActionPlannerSystem;

impl<'a> System<'a> for AIActionPlannerSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Soldier>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, TimeBudget>,
        WriteStorage<'a, QueuedAction>,
        WriteStorage<'a, PlannedPath>,
        Read<'a, Battlefield>,
        Read<'a, TurnState>,
    );

    fn run(
        &mut self,
        (entities, positions, _soldiers, players, mut budgets, mut queued, mut paths, battlefield, turn_state): Self::SystemData,
    ) {
        // Only plan during Planning phase
        if !matches!(turn_state.phase, TurnPhase::Planning) {
            return;
        }

        // For PlayerFirst mode: only plan for NPCs after player is ready
        if matches!(turn_state.turn_order_mode, TurnOrderMode::PlayerFirst) {
            // Check if player entity is ready
            let player_ready = (&entities, &players)
                .join()
                .any(|(e, _)| turn_state.is_entity_ready(e));

            if !player_ready {
                return; // Wait for player to finish
            }
        }

        // Get player position for AI pathfinding
        let player_pos = (&entities, &positions, &players)
            .join()
            .next()
            .map(|(_, pos, _)| pos.as_battlefield_pos().clone());

        // Plan actions for all NPCs (non-player entities with time budgets)
        for (entity, pos, _soldier, budget) in
            (&entities, &positions, &_soldiers, &mut budgets).join()
        {
            // Skip if this is the player
            if players.get(entity).is_some() {
                continue;
            }

            // Skip if entity already has action queued or is out of time
            if queued.get(entity).is_some() || budget.available_time() <= 0.0 {
                continue;
            }

            // Simple AI: move toward player using pathfinding
            if let Some(target_pos) = &player_pos {
                let entity_pos = pos.as_battlefield_pos();

                // Only pathfind if not already adjacent to player
                if entity_pos.distance_to(target_pos) > 1.5 {
                    if let Some(path_steps) = calculate_path(entity_pos, target_pos, &battlefield) {
                        // Insert PlannedPath component (no preview for AI)
                        paths
                            .insert(entity, PlannedPath::new(path_steps, 0.0, false))
                            .ok();
                        continue; // PathExecutionSystem will handle the movement
                    }
                }
            }

            // Fallback: Wait if no valid path or already at player
            let action = ActionType::Wait;
            let time_cost = action.base_time_cost();

            budget.consume_time(time_cost);
            queued
                .insert(
                    entity,
                    QueuedAction {
                        action_type: action,
                        time_cost,
                        committed: true,
                    },
                )
                .ok();
        }
    }
}
