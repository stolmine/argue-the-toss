// Turn Manager System
// Manages turn flow and phase transitions

use crate::components::{
    action::QueuedAction, player::Player, time_budget::TimeBudget,
};
use crate::game_logic::turn_state::{TurnOrderMode, TurnPhase, TurnState};
use crate::utils::event_log::EventLog;
use specs::{Entities, Join, ReadStorage, System, Write, WriteStorage};

pub struct TurnManagerSystem;

impl<'a> System<'a> for TurnManagerSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, TurnState>,
        WriteStorage<'a, TimeBudget>,
        WriteStorage<'a, QueuedAction>,
        ReadStorage<'a, Player>,
        Write<'a, EventLog>,
    );

    fn run(
        &mut self,
        (entities, mut turn_state, mut budgets, mut actions, players, mut log): Self::SystemData,
    ) {
        match turn_state.phase {
            TurnPhase::Planning => {
                // Check if all entities are ready to execute
                let all_ready = match turn_state.turn_order_mode {
                    TurnOrderMode::PlayerFirst => {
                        // In PlayerFirst: check if player is ready, then if all NPCs are ready
                        let player_ready = (&entities, &players)
                            .join()
                            .any(|(e, _)| turn_state.is_entity_ready(e));

                        if !player_ready {
                            return; // Wait for player
                        }

                        // Player ready, check if all NPCs have actions or are out of budget
                        (&entities, &budgets)
                            .join()
                            .filter(|(e, _)| players.get(*e).is_none()) // NPCs only
                            .all(|(e, budget)| {
                                actions.get(e).is_some() || budget.available_time() <= 0.0
                            })
                    }
                    TurnOrderMode::Simultaneous => {
                        // All entities must be ready
                        (&entities, &budgets).join().all(|(e, budget)| {
                            turn_state.is_entity_ready(e) || budget.available_time() <= 0.0
                        })
                    }
                    TurnOrderMode::InitiativeBased => {
                        // Not implemented yet
                        false
                    }
                };

                if all_ready {
                    turn_state.phase = TurnPhase::Execution;
                    log.add("=== Executing Turn ===".to_string());
                }
            }

            TurnPhase::Execution => {
                // Actions have been executed, transition to Resolution
                turn_state.phase = TurnPhase::Resolution;
            }

            TurnPhase::Resolution => {
                // Clear executed actions
                actions.clear();

                // Reset time budgets for new turn (keep debt)
                for budget in (&mut budgets).join() {
                    budget.reset_for_new_turn();
                }

                // Start new turn
                turn_state.reset_for_new_turn();
                log.add(format!("=== Turn {} ===", turn_state.current_turn));
            }
        }
    }
}
