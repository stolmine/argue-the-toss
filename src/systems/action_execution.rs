// Action Execution System
// Executes committed actions during Execution phase
//
// CRITICAL DEPENDENCY: This system MUST run AFTER TurnManagerSystem!
// TurnManagerSystem handles phase transitions (Planning -> Execution -> Resolution).
// If ActionExecutionSystem runs before TurnManagerSystem, it will see the OLD phase
// and skip execution, causing the "movement bug" where actions are logged but entities
// don't move on screen.
//
// Correct execution order (in a single dispatcher.dispatch() call):
// 1. TurnManagerSystem transitions Planning -> Execution
// 2. ActionExecutionSystem sees Execution phase and executes actions
//
// Incorrect order causes:
// 1. ActionExecutionSystem sees Planning phase, returns early
// 2. TurnManagerSystem transitions Planning -> Execution (too late!)
//
// This system also requires WriteStorage<Position> to update entity positions.
// If Position is accidentally changed to ReadStorage, moves will not apply.

use crate::components::{
    action::{ActionType, OngoingAction, QueuedAction},
    dead::Dead,
    facing::Facing,
    health::Health,
    position::Position,
    soldier::Soldier,
    soldier_stats::SoldierStats,
    vision::Vision,
    weapon::Weapon,
};
use crate::game_logic::battlefield::Battlefield;
use crate::game_logic::combat::{apply_damage, calculate_shot};
use crate::game_logic::turn_state::{TurnPhase, TurnState};
use crate::utils::event_log::EventLog;
use specs::{Entities, Join, Read, ReadStorage, System, Write, WriteStorage};

pub struct ActionExecutionSystem;

impl<'a> System<'a> for ActionExecutionSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, QueuedAction>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Facing>,
        WriteStorage<'a, OngoingAction>,
        WriteStorage<'a, Weapon>,
        WriteStorage<'a, Health>,
        WriteStorage<'a, Dead>,
        ReadStorage<'a, Vision>,
        ReadStorage<'a, Soldier>,
        ReadStorage<'a, SoldierStats>,
        Write<'a, EventLog>,
        Read<'a, TurnState>,
        Read<'a, Battlefield>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut queued,
            mut positions,
            mut facings,
            mut _ongoing,
            mut weapons,
            mut healths,
            mut dead_markers,
            visions,
            soldiers,
            soldier_stats,
            mut log,
            turn_state,
            battlefield,
        ): Self::SystemData,
    ) {
        // Only execute during Execution phase
        if !matches!(turn_state.phase, TurnPhase::Execution) {
            return;
        }

        // Execute ALL committed actions (player, allies, enemies)
        for (entity, action) in (&entities, &queued).join() {
            if !action.committed {
                continue;
            }

            // Skip if entity is dead
            if dead_markers.get(entity).is_some() {
                continue;
            }

            match &action.action_type {
                ActionType::Move {
                    dx,
                    dy,
                    terrain_cost: _,
                } => {
                    if let Some(pos) = positions.get_mut(entity) {
                        let old_x = pos.x();
                        let old_y = pos.y();
                        let new_x = old_x + dx;
                        let new_y = old_y + dy;
                        // Boundary check (from battlefield size)
                        if new_x >= 0 && new_x < 100 && new_y >= 0 && new_y < 100 {
                            *pos = Position::new(new_x, new_y);
                            if let Some(soldier) = soldiers.get(entity) {
                                log.add(format!("{} moved from ({}, {}) to ({}, {})",
                                    soldier.name, old_x, old_y, new_x, new_y));
                            } else {
                                log.add(format!("Entity moved from ({}, {}) to ({}, {})",
                                    old_x, old_y, new_x, new_y));
                            }
                        } else {
                            log.add(format!("Move blocked: ({}, {}) out of bounds", new_x, new_y));
                        }
                    } else {
                        log.add("Move failed: entity has no position component".to_string());
                    }
                }
                ActionType::Rotate { clockwise } => {
                    // Execute rotation
                    if let Some(facing) = facings.get_mut(entity) {
                        if *clockwise {
                            facing.rotate_cw();
                        } else {
                            facing.rotate_ccw();
                        }

                        if let Some(soldier) = soldiers.get(entity) {
                            log.add(format!("{} rotates {}.", soldier.name,
                                if *clockwise { "clockwise" } else { "counter-clockwise" }));
                        }
                    }
                }
                ActionType::Wait => {
                    // Waiting is a no-op execution
                }
                ActionType::Shoot { target } => {
                    // Execute shooting action
                    execute_shoot(
                        entity,
                        *target,
                        &positions,
                        &mut weapons,
                        &mut healths,
                        &mut dead_markers,
                        &visions,
                        &soldiers,
                        &soldier_stats,
                        &mut log,
                        &battlefield,
                    );
                }
                ActionType::Reload => {
                    // Execute reload action
                    if let Some(weapon) = weapons.get_mut(entity) {
                        weapon.reload();
                        if let Some(soldier) = soldiers.get(entity) {
                            log.add(format!("{} reloads.", soldier.name));
                        } else {
                            log.add("Entity reloads.".to_string());
                        }
                    }
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

        // Remove executed actions
        let mut to_remove = Vec::new();
        for (entity, action) in (&entities, &queued).join() {
            if action.committed {
                to_remove.push(entity);
            }
        }

        for entity in to_remove {
            queued.remove(entity);
        }
    }
}

/// Execute a shooting action from shooter to target
fn execute_shoot(
    shooter: specs::Entity,
    target: specs::Entity,
    positions: &WriteStorage<Position>,
    weapons: &mut WriteStorage<Weapon>,
    healths: &mut WriteStorage<Health>,
    dead_markers: &mut WriteStorage<Dead>,
    visions: &ReadStorage<Vision>,
    soldiers: &ReadStorage<Soldier>,
    soldier_stats: &ReadStorage<SoldierStats>,
    log: &mut EventLog,
    battlefield: &Battlefield,
) {
    // Get shooter's weapon
    let shooter_weapon = match weapons.get_mut(shooter) {
        Some(weapon) => weapon,
        None => {
            log.add("Shooter has no weapon!".to_string());
            return;
        }
    };

    // Check if weapon has ammo
    if !shooter_weapon.can_fire() {
        if let Some(soldier) = soldiers.get(shooter) {
            log.add(format!("{} is out of ammo!", soldier.name));
        } else {
            log.add("Out of ammo!".to_string());
        }
        return;
    }

    // Get positions
    let shooter_pos = match positions.get(shooter) {
        Some(pos) => pos,
        None => return,
    };

    let target_pos = match positions.get(target) {
        Some(pos) => pos,
        None => {
            log.add("Target not found!".to_string());
            return;
        }
    };

    // Get shooter vision for LOS check
    let shooter_vision = visions.get(shooter).map(|v| v.range).unwrap_or(10);

    // Get shooter accuracy modifier from stats
    let shooter_accuracy = soldier_stats.get(shooter).map(|stats| stats.accuracy_modifier);

    // Calculate shot result
    let result = calculate_shot(
        shooter_weapon,
        shooter_pos,
        target_pos,
        battlefield,
        shooter_vision,
        shooter_accuracy,
    );

    // Consume ammo
    shooter_weapon.fire();

    // Get names for logging
    let shooter_name = soldiers
        .get(shooter)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "Entity".to_string());
    let target_name = soldiers
        .get(target)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "Target".to_string());

    // Handle result
    if result.blocked_by_los {
        log.add(format!(
            "{} shoots at {} but has no line of sight!",
            shooter_name, target_name
        ));
    } else if result.hit {
        // Apply damage to target
        if let Some(target_health) = healths.get_mut(target) {
            let still_alive = apply_damage(target_health, result.damage);
            if still_alive {
                log.add(format!(
                    "{} shoots {} for {} damage! ({} HP remaining)",
                    shooter_name, target_name, result.damage, target_health.current
                ));
            } else {
                log.add(format!(
                    "{} shoots {} for {} damage! {} is killed!",
                    shooter_name, target_name, result.damage, target_name
                ));
                // Mark target as dead
                dead_markers.insert(target, Dead).ok();
            }
        } else {
            log.add(format!("{} shoots {} and hits!", shooter_name, target_name));
        }
    } else {
        log.add(format!(
            "{} shoots at {} and misses! ({}% chance, {} tiles)",
            shooter_name,
            target_name,
            (result.hit_chance * 100.0) as i32,
            result.distance
        ));
    }
}
