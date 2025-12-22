use crate::components::{
    action::ActionType, facing::Facing, position::Position, soldier::Soldier, weapon::Weapon,
};
use crate::game_logic::battlefield::{Battlefield, Position as BattlefieldPos};
use crate::game_logic::objectives::Objectives;
use specs::{Entity, ReadStorage};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct PossibleAction {
    pub action_type: ActionType,
    pub target_entity: Option<Entity>,
    pub target_position: Option<BattlefieldPos>,
}

impl PossibleAction {
    pub fn new(action_type: ActionType) -> Self {
        Self {
            action_type,
            target_entity: None,
            target_position: None,
        }
    }

    pub fn with_target(mut self, target: Entity) -> Self {
        self.target_entity = Some(target);
        self
    }

    pub fn with_position(mut self, pos: BattlefieldPos) -> Self {
        self.target_position = Some(pos);
        self
    }
}

pub struct ActionGenerator;

impl ActionGenerator {
    pub fn generate_actions(
        actor_entity: Entity,
        visible_enemies: &[Entity],
        positions: &ReadStorage<Position>,
        soldiers: &ReadStorage<Soldier>,
        weapons: &ReadStorage<Weapon>,
        battlefield: &Battlefield,
        objectives: &Objectives,
    ) -> Vec<PossibleAction> {
        let mut actions = Vec::new();

        let actor_pos = match positions.get(actor_entity) {
            Some(pos) => pos,
            None => return actions,
        };

        let actor_weapon = weapons.get(actor_entity);
        let actor_faction = soldiers.get(actor_entity).map(|s| s.faction);

        actions.extend(Self::generate_shoot_actions(
            visible_enemies,
            actor_pos,
            actor_weapon,
            positions,
            battlefield,
        ));

        actions.extend(Self::generate_reload_action(actor_weapon));

        actions.extend(Self::generate_move_actions(
            actor_pos,
            visible_enemies,
            positions,
            battlefield,
            objectives,
            actor_faction,
        ));

        actions.extend(Self::generate_rotation_actions());

        actions.push(PossibleAction::new(ActionType::Wait));

        actions
    }

    fn generate_shoot_actions(
        visible_enemies: &[Entity],
        actor_pos: &Position,
        actor_weapon: Option<&Weapon>,
        positions: &ReadStorage<Position>,
        battlefield: &Battlefield,
    ) -> Vec<PossibleAction> {
        let mut actions = Vec::new();

        let weapon = match actor_weapon {
            Some(w) => w,
            None => return actions,
        };

        if !weapon.can_fire() {
            return actions;
        }

        for &enemy in visible_enemies {
            if let Some(enemy_pos) = positions.get(enemy) {
                let distance = actor_pos.as_battlefield_pos().distance_to(enemy_pos.as_battlefield_pos());

                if distance <= weapon.stats.max_range as f32 {
                    actions.push(
                        PossibleAction::new(ActionType::Shoot { target: enemy })
                            .with_target(enemy)
                            .with_position(*enemy_pos.as_battlefield_pos()),
                    );
                }
            }
        }

        actions
    }

    fn generate_reload_action(actor_weapon: Option<&Weapon>) -> Vec<PossibleAction> {
        let mut actions = Vec::new();

        if let Some(weapon) = actor_weapon {
            if weapon.ammo.current < weapon.ammo.max_capacity {
                actions.push(PossibleAction::new(ActionType::Reload));
            }
        }

        actions
    }

    fn generate_move_actions(
        actor_pos: &Position,
        visible_enemies: &[Entity],
        positions: &ReadStorage<Position>,
        battlefield: &Battlefield,
        objectives: &Objectives,
        actor_faction: Option<crate::components::soldier::Faction>,
    ) -> Vec<PossibleAction> {
        let mut actions = Vec::new();
        let mut target_positions = HashSet::new();

        if !visible_enemies.is_empty() {
            if let Some(nearest_enemy_pos) = Self::find_nearest_enemy(actor_pos, visible_enemies, positions) {
                target_positions.extend(Self::sample_positions_toward(
                    actor_pos.as_battlefield_pos(),
                    &nearest_enemy_pos,
                    battlefield,
                    3,
                ));

                target_positions.extend(Self::calculate_flanking_positions(
                    actor_pos.as_battlefield_pos(),
                    &nearest_enemy_pos,
                    None,
                ));
            }
        }

        if let Some(faction) = actor_faction {
            if let Some(objective_pos) = objectives.get_enemy_flag_position(faction) {
                target_positions.extend(Self::sample_positions_toward(
                    actor_pos.as_battlefield_pos(),
                    &objective_pos,
                    battlefield,
                    3,
                ));
            }
        }

        target_positions.extend(Self::sample_cover_positions(
            actor_pos.as_battlefield_pos(),
            battlefield,
            8,
        ));

        for target_pos in target_positions {
            if let Some(action) = Self::create_move_action(actor_pos.as_battlefield_pos(), &target_pos, battlefield) {
                actions.push(action);
            }
        }

        if actions.len() > 15 {
            actions.truncate(15);
        }

        actions
    }

    fn generate_rotation_actions() -> Vec<PossibleAction> {
        vec![
            PossibleAction::new(ActionType::Rotate { clockwise: true }),
            PossibleAction::new(ActionType::Rotate { clockwise: false }),
        ]
    }

    fn find_nearest_enemy(
        actor_pos: &Position,
        visible_enemies: &[Entity],
        positions: &ReadStorage<Position>,
    ) -> Option<BattlefieldPos> {
        visible_enemies
            .iter()
            .filter_map(|&enemy| positions.get(enemy).map(|pos| pos.as_battlefield_pos()))
            .min_by(|a, b| {
                let dist_a = actor_pos.as_battlefield_pos().distance_to(a);
                let dist_b = actor_pos.as_battlefield_pos().distance_to(b);
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }

    fn sample_positions_toward(
        from: &BattlefieldPos,
        toward: &BattlefieldPos,
        battlefield: &Battlefield,
        sample_count: usize,
    ) -> Vec<BattlefieldPos> {
        let mut positions = Vec::new();
        let dx = toward.x - from.x;
        let dy = toward.y - from.y;
        let distance = from.distance_to(toward);

        if distance < 0.1 {
            return positions;
        }

        let step_x = dx as f32 / distance;
        let step_y = dy as f32 / distance;

        for i in 1..=sample_count {
            let factor = i as f32 * 2.0;
            let new_x = from.x + (step_x * factor).round() as i32;
            let new_y = from.y + (step_y * factor).round() as i32;
            let pos = BattlefieldPos::new(new_x, new_y);

            if battlefield.in_bounds(&pos) {
                if let Some(tile) = battlefield.get_tile(&pos) {
                    if tile.terrain.is_passable() {
                        positions.push(pos);
                    }
                }
            }
        }

        positions
    }

    fn calculate_flanking_positions(
        actor_pos: &BattlefieldPos,
        target_pos: &BattlefieldPos,
        _target_facing: Option<&Facing>,
    ) -> Vec<BattlefieldPos> {
        let mut positions = Vec::new();

        let dx = target_pos.x - actor_pos.x;
        let dy = target_pos.y - actor_pos.y;

        let perpendicular_offsets = [(-dy, dx), (dy, -dx)];

        for (offset_x, offset_y) in perpendicular_offsets.iter() {
            let flank_x = target_pos.x + offset_x.signum() * 3;
            let flank_y = target_pos.y + offset_y.signum() * 3;
            positions.push(BattlefieldPos::new(flank_x, flank_y));
        }

        positions
    }

    fn sample_cover_positions(
        current_pos: &BattlefieldPos,
        battlefield: &Battlefield,
        sample_count: usize,
    ) -> Vec<BattlefieldPos> {
        let mut positions = Vec::new();
        let search_radius = 5;

        let mut candidates = Vec::new();
        for dy in -search_radius..=search_radius {
            for dx in -search_radius..=search_radius {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let pos = BattlefieldPos::new(current_pos.x + dx, current_pos.y + dy);

                if !battlefield.in_bounds(&pos) {
                    continue;
                }

                if let Some(tile) = battlefield.get_tile(&pos) {
                    if !tile.terrain.is_passable() {
                        continue;
                    }

                    let cover_bonus = tile.terrain.cover_bonus();
                    if cover_bonus > 0.1 {
                        candidates.push((pos, cover_bonus));
                    }
                }
            }
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (pos, _) in candidates.into_iter().take(sample_count) {
            positions.push(pos);
        }

        positions
    }

    fn create_move_action(
        from: &BattlefieldPos,
        to: &BattlefieldPos,
        battlefield: &Battlefield,
    ) -> Option<PossibleAction> {
        let dx = to.x - from.x;
        let dy = to.y - from.y;

        if dx == 0 && dy == 0 {
            return None;
        }

        let terrain_cost = battlefield
            .get_tile(to)
            .map(|t| t.terrain.movement_cost())
            .unwrap_or(1.0);

        Some(
            PossibleAction::new(ActionType::Move { dx, dy, terrain_cost })
                .with_position(*to),
        )
    }
}
