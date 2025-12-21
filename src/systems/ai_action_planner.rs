// AI Action Planning System
// Generates actions for NPC entities

use crate::components::{
    action::{ActionType, QueuedAction},
    player::Player,
    position::Position,
    soldier::Soldier,
    time_budget::TimeBudget,
};
use crate::game_logic::turn_state::{TurnOrderMode, TurnPhase, TurnState};
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
        Read<'a, TurnState>,
    );

    fn run(
        &mut self,
        (entities, _positions, _soldiers, players, mut budgets, mut queued, turn_state): Self::SystemData,
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

        // Plan actions for all NPCs (non-player entities with time budgets)
        for (entity, _pos, _soldier, budget) in
            (&entities, &_positions, &_soldiers, &mut budgets).join()
        {
            // Skip if this is the player
            if players.get(entity).is_some() {
                continue;
            }

            // Skip if entity already has action queued or is out of time
            if queued.get(entity).is_some() || budget.available_time() <= 0.0 {
                continue;
            }

            // Simple AI: just wait for now (future: pathfinding, combat decisions)
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
