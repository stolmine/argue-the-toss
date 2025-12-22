// Turn Manager System
// Manages turn flow and phase transitions
//
// CRITICAL SYSTEM ORDER DEPENDENCY:
// This system MUST run BEFORE ActionExecutionSystem in the dispatcher!
//
// This system manages the turn state machine:
// - Planning: Entities queue actions, AI plans, player inputs commands
// - Execution: ActionExecutionSystem executes all committed actions
// - Resolution: Cleanup and preparation for next turn
//
// Phase transitions happen by mutating TurnState.phase. Since all systems in a
// dispatcher.dispatch() call share the same world resources, systems that run
// AFTER this one will see the updated phase in the SAME frame.
//
// Execution flow (single dispatch() call):
// 1. TurnManagerSystem (Planning) -> transitions to Execution
// 2. ActionExecutionSystem sees Execution phase -> executes actions
// 3. TurnManagerSystem (next frame, Execution) -> transitions to Resolution
// 4. TurnManagerSystem (next frame, Resolution) -> clears and starts new turn
//
// If ActionExecutionSystem runs before this system, it will see the OLD phase
// and fail to execute, causing the "movement bug."

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
                // Check if there are any committed actions left to execute
                // If no committed actions remain, move to Resolution
                let has_committed_actions = (&actions).join().any(|action| action.committed);

                if !has_committed_actions {
                    turn_state.phase = TurnPhase::Resolution;
                }
                // Otherwise, stay in Execution phase and let actions execute
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
