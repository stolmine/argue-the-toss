// AI Action Planning System
// Generates actions for NPC entities using utility-based AI

use crate::ai::{
    actions::{
        create_move_evaluator, create_reload_evaluator, create_seek_cover_evaluator,
        create_seek_objective_evaluator, create_shoot_evaluator, create_wait_evaluator,
        ActionEvaluator, ScoredAction,
    },
    considerations::ActionContext,
    personality::AIPersonality,
    ActionGenerator, PossibleAction,
};
use crate::components::{
    action::{ActionType, QueuedAction},
    dead::Dead,
    facing::Facing,
    health::Health,
    pathfinding::PlannedPath,
    player::Player,
    position::Position,
    soldier::{Faction, Rank, Soldier},
    time_budget::TimeBudget,
    vision::Vision,
    weapon::Weapon,
};
use crate::game_logic::{
    battlefield::Battlefield,
    line_of_sight::calculate_fov,
    objectives::Objectives,
    pathfinding::calculate_path,
    turn_state::{TurnOrderMode, TurnPhase, TurnState},
};
use crate::utils::event_log::EventLog;
use specs::{Entities, Entity, Join, Read, ReadStorage, System, Write, WriteStorage};
pub struct AIActionPlannerSystem;

impl AIActionPlannerSystem {
    pub fn new() -> Self {
        Self
    }

    fn get_evaluators(&self, rank: Rank) -> Vec<ActionEvaluator> {
        let personality = self.get_personality_for_rank(rank);
        personality.evaluators
    }

    fn get_personality_for_rank(&self, rank: Rank) -> AIPersonality {
        match rank {
            Rank::Captain => AIPersonality::objective_focused(),
            Rank::Lieutenant => AIPersonality::aggressive(),
            Rank::Sergeant => AIPersonality::balanced(),
            Rank::Corporal => AIPersonality::balanced(),
            Rank::Private => AIPersonality::defensive(),
        }
    }

    fn calculate_visible_enemies(
        &self,
        entity: Entity,
        pos: &Position,
        soldier: &Soldier,
        entities: &Entities,
        positions: &ReadStorage<Position>,
        soldiers: &ReadStorage<Soldier>,
        healths: &ReadStorage<Health>,
        visions: &ReadStorage<Vision>,
        battlefield: &Battlefield,
    ) -> Vec<Entity> {
        let ai_faction = soldier.faction;
        let vision_range = visions.get(entity).map(|v| v.range).unwrap_or(10);
        let visible_tiles = calculate_fov(&pos.as_battlefield_pos(), vision_range, battlefield);

        (entities, positions, soldiers, healths)
            .join()
            .filter(|(e, _, _, _)| *e != entity)
            .filter(|(_, target_pos, target_soldier, target_health)| {
                target_soldier.faction != ai_faction
                    && target_health.is_alive()
                    && visible_tiles.contains(&target_pos.as_battlefield_pos())
            })
            .map(|(e, _, _, _)| e)
            .collect()
    }

    fn score_action(
        &self,
        action: &PossibleAction,
        context: &ActionContext,
        evaluators: &Vec<ActionEvaluator>,
    ) -> f32 {
        let mut max_score: f32 = 0.0;
        let mut matched = false;

        for evaluator in evaluators {
            if self.evaluator_matches_action(&evaluator.name, &action.action_type) {
                let score = evaluator.evaluate(context);
                max_score = max_score.max(score);
                matched = true;
            }
        }

        if !matched {
            return 0.0;
        }

        max_score
    }

    fn evaluator_matches_action(&self, evaluator_name: &str, action_type: &ActionType) -> bool {
        match action_type {
            ActionType::Shoot { .. } => evaluator_name.contains("Shoot"),
            ActionType::Reload => evaluator_name.contains("Reload"),
            ActionType::Move { .. } => {
                evaluator_name.contains("Move")
                    || evaluator_name.contains("Cover")
                    || evaluator_name.contains("Objective")
            }
            ActionType::Rotate { .. } => evaluator_name.contains("Rotate"),
            ActionType::Wait => evaluator_name.contains("Wait"),
            _ => false,
        }
    }

    fn queue_action(
        &self,
        entity: Entity,
        action: &ScoredAction,
        queued: &mut WriteStorage<QueuedAction>,
        budget: &mut TimeBudget,
        event_log: &mut EventLog,
        soldier_name: Option<&str>,
    ) {
        let time_cost = action.action_type.base_time_cost();

        budget.consume_time(time_cost);
        queued
            .insert(
                entity,
                QueuedAction {
                    action_type: action.action_type.clone(),
                    time_cost,
                    committed: true,
                },
            )
            .ok();

        if cfg!(debug_assertions) {
            if let Some(name) = soldier_name {
                event_log.add(format!(
                    "AI {}: {:?} (score: {:.2})",
                    name,
                    action.action_type,
                    action.score
                ));
            }
        }
    }

    fn queue_move_action(
        &self,
        entity: Entity,
        target_pos: &crate::game_logic::battlefield::Position,
        current_pos: &Position,
        battlefield: &Battlefield,
        queued: &mut WriteStorage<QueuedAction>,
        budget: &mut TimeBudget,
    ) -> bool {
        let dx = target_pos.x - current_pos.x();
        let dy = target_pos.y - current_pos.y();

        if dx == 0 && dy == 0 {
            return false;
        }

        let terrain_cost = battlefield
            .get_tile(target_pos)
            .map(|t| t.terrain.movement_cost())
            .unwrap_or(1.0);

        let action = ActionType::Move {
            dx,
            dy,
            terrain_cost,
        };
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

        true
    }
}

impl<'a> System<'a> for AIActionPlannerSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Soldier>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Vision>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, Dead>,
        ReadStorage<'a, Weapon>,
        ReadStorage<'a, Facing>,
        WriteStorage<'a, TimeBudget>,
        WriteStorage<'a, QueuedAction>,
        WriteStorage<'a, PlannedPath>,
        Read<'a, Battlefield>,
        Read<'a, TurnState>,
        Read<'a, Objectives>,
        Write<'a, EventLog>,
    );

    fn run(
        &mut self,
        (
            entities,
            positions,
            soldiers,
            players,
            visions,
            healths,
            dead_markers,
            weapons,
            facings,
            mut budgets,
            mut queued,
            mut paths,
            battlefield,
            turn_state,
            objectives,
            mut event_log,
        ): Self::SystemData,
    ) {
        if !matches!(turn_state.phase, TurnPhase::Planning) {
            return;
        }

        if matches!(turn_state.turn_order_mode, TurnOrderMode::PlayerFirst) {
            let player_ready = (&entities, &players)
                .join()
                .any(|(e, _)| turn_state.is_entity_ready(e));

            if !player_ready {
                return;
            }
        }

        for (entity, pos, soldier, budget) in (&entities, &positions, &soldiers, &mut budgets).join()
        {
            if players.get(entity).is_some() {
                continue;
            }

            if dead_markers.get(entity).is_some() {
                continue;
            }

            if queued.get(entity).is_some() || budget.available_time() <= 0.0 {
                continue;
            }

            let visible_enemies = self.calculate_visible_enemies(
                entity,
                pos,
                soldier,
                &entities,
                &positions,
                &soldiers,
                &healths,
                &visions,
                &battlefield,
            );

            let possible_actions = ActionGenerator::generate_actions(
                entity,
                &visible_enemies,
                &positions,
                &soldiers,
                &weapons,
                &battlefield,
                &objectives,
            );

            let evaluators = self.get_evaluators(soldier.rank);

            let mut scored_actions = Vec::new();
            for possible_action in &possible_actions {
                let context = ActionContext {
                    actor_entity: entity,
                    target_entity: possible_action.target_entity,
                    target_position: possible_action.target_position,
                    positions: &positions,
                    soldiers: &soldiers,
                    healths: &healths,
                    weapons: &weapons,
                    visions: &visions,
                    facings: &facings,
                    battlefield: &battlefield,
                    objectives: &objectives,
                    entities: &entities,
                    visible_enemies: &visible_enemies,
                };

                let score = self.score_action(&possible_action, &context, &evaluators);

                scored_actions.push(ScoredAction {
                    action_type: possible_action.action_type.clone(),
                    target: possible_action.target_entity,
                    position: possible_action.target_position,
                    score,
                    debug_info: None,
                });
            }

            if cfg!(debug_assertions) && !scored_actions.is_empty() {
                event_log.add(format!(
                    "{} considering {} actions (enemies: {})",
                    soldier.name,
                    scored_actions.len(),
                    visible_enemies.len()
                ));
            }

            if let Some(best_action) = scored_actions
                .iter()
                .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
            {
                match &best_action.action_type {
                    ActionType::Move { .. } => {
                        if let Some(target_pos) = &best_action.position {
                            let ai_pos = pos.as_battlefield_pos();
                            if ai_pos.distance_to(target_pos) > 1.5 {
                                if let Some(path_steps) =
                                    calculate_path(ai_pos, target_pos, &battlefield)
                                {
                                    paths
                                        .insert(entity, PlannedPath::new(path_steps, 0.0, false))
                                        .ok();
                                } else if ai_pos.distance_to(target_pos) <= 1.5 {
                                    self.queue_move_action(
                                        entity,
                                        target_pos,
                                        pos,
                                        &battlefield,
                                        &mut queued,
                                        budget,
                                    );
                                }
                            } else {
                                self.queue_move_action(
                                    entity,
                                    target_pos,
                                    pos,
                                    &battlefield,
                                    &mut queued,
                                    budget,
                                );
                            }
                        }
                    }
                    _ => {
                        self.queue_action(
                            entity,
                            best_action,
                            &mut queued,
                            budget,
                            &mut event_log,
                            Some(&soldier.name),
                        );
                    }
                }
            }
        }
    }
}

impl Default for AIActionPlannerSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::action::ActionType;

    #[test]
    fn test_evaluator_matches_move_action() {
        let system = AIActionPlannerSystem::new();
        let move_action = ActionType::Move { dx: 1, dy: 0, terrain_cost: 1.0 };

        // Move actions should match Move, Cover, and Objective evaluators
        // This is the key fix - all three evaluators should be able to score move actions
        assert!(system.evaluator_matches_action("Move", &move_action));
        assert!(system.evaluator_matches_action("SeekCover", &move_action));
        assert!(system.evaluator_matches_action("SeekObjective", &move_action));
        assert!(!system.evaluator_matches_action("Shoot", &move_action));
        assert!(!system.evaluator_matches_action("Wait", &move_action));
    }

    #[test]
    fn test_evaluator_matches_wait_action() {
        let system = AIActionPlannerSystem::new();
        let wait_action = ActionType::Wait;

        assert!(system.evaluator_matches_action("Wait", &wait_action));
        assert!(!system.evaluator_matches_action("Move", &wait_action));
    }

    #[test]
    fn test_evaluator_matches_reload_action() {
        let system = AIActionPlannerSystem::new();
        let reload_action = ActionType::Reload;

        assert!(system.evaluator_matches_action("Reload", &reload_action));
        assert!(!system.evaluator_matches_action("Move", &reload_action));
    }

    #[test]
    fn test_rank_based_personality_assignment() {
        let system = AIActionPlannerSystem::new();

        let captain_personality = system.get_personality_for_rank(Rank::Captain);
        assert_eq!(captain_personality.name, "ObjectiveFocused");

        let lieutenant_personality = system.get_personality_for_rank(Rank::Lieutenant);
        assert_eq!(lieutenant_personality.name, "Aggressive");

        let sergeant_personality = system.get_personality_for_rank(Rank::Sergeant);
        assert_eq!(sergeant_personality.name, "Balanced");

        let corporal_personality = system.get_personality_for_rank(Rank::Corporal);
        assert_eq!(corporal_personality.name, "Balanced");

        let private_personality = system.get_personality_for_rank(Rank::Private);
        assert_eq!(private_personality.name, "Defensive");
    }

    #[test]
    fn test_get_evaluators_returns_personality_evaluators() {
        let system = AIActionPlannerSystem::new();

        let captain_evaluators = system.get_evaluators(Rank::Captain);
        assert_eq!(captain_evaluators.len(), 6);

        let private_evaluators = system.get_evaluators(Rank::Private);
        assert_eq!(private_evaluators.len(), 6);
    }
}
