// Action Execution System
// Executes committed actions during Execution phase

use crate::components::{
    action::{ActionType, OngoingAction, QueuedAction},
    dead::Dead,
    health::Health,
    position::Position,
    soldier::Soldier,
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
        ReadStorage<'a, QueuedAction>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, OngoingAction>,
        WriteStorage<'a, Weapon>,
        WriteStorage<'a, Health>,
        WriteStorage<'a, Dead>,
        ReadStorage<'a, Vision>,
        ReadStorage<'a, Soldier>,
        Write<'a, EventLog>,
        Read<'a, TurnState>,
        Read<'a, Battlefield>,
    );

    fn run(
        &mut self,
        (
            entities,
            queued,
            mut positions,
            mut _ongoing,
            mut weapons,
            mut healths,
            mut dead_markers,
            visions,
            soldiers,
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

            match &action.action_type {
                ActionType::Move {
                    dx,
                    dy,
                    terrain_cost: _,
                } => {
                    if let Some(pos) = positions.get_mut(entity) {
                        let new_x = pos.x() + dx;
                        let new_y = pos.y() + dy;
                        // Boundary check (from battlefield size)
                        if new_x >= 0 && new_x < 100 && new_y >= 0 && new_y < 100 {
                            *pos = Position::new(new_x, new_y);
                            log.add(format!("Entity moved to ({}, {})", new_x, new_y));
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

        // Clear executed actions after processing
        // (done by cleanup system in turn manager)
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

    // Calculate shot result
    let result = calculate_shot(
        shooter_weapon,
        shooter_pos,
        target_pos,
        battlefield,
        shooter_vision,
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
