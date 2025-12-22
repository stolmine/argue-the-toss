// AI Action Planning System
// Generates actions for NPC entities

use crate::components::{
    action::{ActionType, QueuedAction},
    dead::Dead,
    health::Health,
    pathfinding::PlannedPath,
    player::Player,
    position::Position,
    soldier::Soldier,
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
use specs::{Entities, Entity, Join, Read, ReadStorage, System, WriteStorage};

pub struct AIActionPlannerSystem;

impl<'a> System<'a> for AIActionPlannerSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Soldier>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Vision>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, Dead>,
        WriteStorage<'a, Weapon>,
        WriteStorage<'a, TimeBudget>,
        WriteStorage<'a, QueuedAction>,
        WriteStorage<'a, PlannedPath>,
        Read<'a, Battlefield>,
        Read<'a, TurnState>,
        Read<'a, Objectives>,
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
            mut weapons,
            mut budgets,
            mut queued,
            mut paths,
            battlefield,
            turn_state,
            objectives,
        ): Self::SystemData,
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
        for (entity, pos, soldier, budget) in (&entities, &positions, &soldiers, &mut budgets).join()
        {
            // Skip if this is the player
            if players.get(entity).is_some() {
                continue;
            }

            // Skip if entity is dead
            if dead_markers.get(entity).is_some() {
                continue;
            }

            // Skip if entity already has action queued or is out of time
            if queued.get(entity).is_some() || budget.available_time() <= 0.0 {
                continue;
            }

            // Extensible AI decision-making: Priority-based action selection
            // This makes it easy to add new behaviors - just add a new decision function!

            // Priority 1: Reload if out of ammo
            if try_reload(entity, &mut weapons, &mut queued, budget) {
                continue;
            }

            // Priority 2: Shoot at visible enemies
            if try_shoot_enemy(
                entity,
                pos,
                soldier,
                &entities,
                &positions,
                &soldiers,
                &healths,
                &visions,
                &mut queued,
                budget,
                &battlefield,
            ) {
                continue;
            }

            // Priority 3: Move toward enemy (find enemies first, then pathfind)
            if try_move_toward_enemy(
                entity,
                pos,
                soldier,
                &entities,
                &positions,
                &soldiers,
                &mut paths,
                &mut queued,
                budget,
                &battlefield,
            ) {
                continue;
            }

            // Priority 4: Move toward enemy objective flag
            if try_move_toward_objective(
                entity,
                pos,
                soldier,
                &mut paths,
                &objectives,
                &battlefield,
            ) {
                continue;
            }

            // Priority 5: Wait (fallback)
            queue_wait_action(entity, &mut queued, budget);
        }
    }
}

/// Extensible AI helper: Try to reload if weapon needs it
fn try_reload(
    entity: Entity,
    weapons: &mut WriteStorage<Weapon>,
    queued: &mut WriteStorage<QueuedAction>,
    budget: &mut TimeBudget,
) -> bool {
    if let Some(weapon) = weapons.get_mut(entity) {
        if weapon.ammo.is_empty() && !weapon.ammo.is_full() {
            let action = ActionType::Reload;
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
            return true;
        }
    }
    false
}

/// Extensible AI helper: Try to shoot at visible enemies
fn try_shoot_enemy(
    entity: Entity,
    pos: &Position,
    soldier: &Soldier,
    entities: &Entities,
    positions: &ReadStorage<Position>,
    soldiers: &ReadStorage<Soldier>,
    healths: &ReadStorage<Health>,
    visions: &ReadStorage<Vision>,
    queued: &mut WriteStorage<QueuedAction>,
    budget: &mut TimeBudget,
    battlefield: &Battlefield,
) -> bool {
    // Get AI's faction
    let ai_faction = soldier.faction;

    // Get vision range
    let vision_range = visions.get(entity).map(|v| v.range).unwrap_or(10);

    // Calculate FOV
    let visible_tiles = calculate_fov(&pos.as_battlefield_pos(), vision_range, battlefield);

    // Find visible enemies (alive and different faction)
    let target = (entities, positions, soldiers, healths)
        .join()
        .filter(|(e, _, _, _)| *e != entity) // Not self
        .filter(|(_, target_pos, target_soldier, target_health)| {
            // Different faction
            target_soldier.faction != ai_faction
                // Alive
                && target_health.is_alive()
                // In FOV
                && visible_tiles.contains(&target_pos.as_battlefield_pos())
        })
        .min_by_key(|(_, target_pos, _, _)| {
            // Pick closest enemy
            let dx = (pos.x() - target_pos.x()).abs();
            let dy = (pos.y() - target_pos.y()).abs();
            dx + dy
        })
        .map(|(e, _, _, _)| e);

    if let Some(target_entity) = target {
        let action = ActionType::Shoot {
            target: target_entity,
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
        return true;
    }

    false
}

/// Extensible AI helper: Try to move toward nearest enemy
fn try_move_toward_enemy(
    entity: Entity,
    pos: &Position,
    soldier: &Soldier,
    entities: &Entities,
    positions: &ReadStorage<Position>,
    soldiers: &ReadStorage<Soldier>,
    paths: &mut WriteStorage<PlannedPath>,
    _queued: &mut WriteStorage<QueuedAction>,
    _budget: &mut TimeBudget,
    battlefield: &Battlefield,
) -> bool {
    let ai_faction = soldier.faction;

    // Find all enemy positions
    let enemy_positions: Vec<_> = (entities, positions, soldiers)
        .join()
        .filter(|(e, _, enemy_soldier)| *e != entity && enemy_soldier.faction != ai_faction)
        .map(|(_, enemy_pos, _)| enemy_pos.as_battlefield_pos().clone())
        .collect();

    if enemy_positions.is_empty() {
        return false;
    }

    // Find closest enemy
    let ai_pos = pos.as_battlefield_pos();
    let closest_enemy = enemy_positions
        .iter()
        .min_by_key(|enemy_pos| {
            let dist = ai_pos.distance_to(enemy_pos);
            (dist * 100.0) as i32 // Convert to int for comparison
        })
        .unwrap();

    // Only pathfind if not already adjacent
    if ai_pos.distance_to(closest_enemy) > 1.5 {
        if let Some(path_steps) = calculate_path(ai_pos, closest_enemy, battlefield) {
            // Insert PlannedPath component (no preview for AI)
            paths
                .insert(entity, PlannedPath::new(path_steps, 0.0, false))
                .ok();
            return true; // PathExecutionSystem will handle the movement
        }
    }

    false
}

/// Extensible AI helper: Try to move toward enemy objective flag
fn try_move_toward_objective(
    entity: Entity,
    pos: &Position,
    soldier: &Soldier,
    paths: &mut WriteStorage<PlannedPath>,
    objectives: &Objectives,
    battlefield: &Battlefield,
) -> bool {
    if let Some(flag_pos) = objectives.get_enemy_flag_position(soldier.faction) {
        let ai_pos = pos.as_battlefield_pos();

        if ai_pos.distance_to(&flag_pos) > 1.5 {
            if let Some(path_steps) = calculate_path(ai_pos, &flag_pos, battlefield) {
                paths
                    .insert(entity, PlannedPath::new(path_steps, 0.0, false))
                    .ok();
                return true;
            }
        }
    }

    false
}

/// Extensible AI helper: Queue a wait action
fn queue_wait_action(
    entity: Entity,
    queued: &mut WriteStorage<QueuedAction>,
    budget: &mut TimeBudget,
) {
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
