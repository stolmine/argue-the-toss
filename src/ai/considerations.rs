use crate::ai::response_curves::ResponseCurve;
use crate::components::{
    facing::Facing, health::Health, position::Position, soldier::{Faction, Rank, Soldier},
    vision::Vision, weapon::Weapon,
};
use crate::game_logic::battlefield::{Battlefield, Position as BattlefieldPos};
use crate::game_logic::line_of_sight::calculate_fov;
use crate::game_logic::objectives::Objectives;
use specs::{Entities, Entity, Join, ReadStorage};

pub struct ActionContext<'a> {
    pub actor_entity: Entity,
    pub target_entity: Option<Entity>,
    pub target_position: Option<BattlefieldPos>,

    pub positions: &'a ReadStorage<'a, Position>,
    pub soldiers: &'a ReadStorage<'a, Soldier>,
    pub healths: &'a ReadStorage<'a, Health>,
    pub weapons: &'a ReadStorage<'a, Weapon>,
    pub visions: &'a ReadStorage<'a, Vision>,
    pub facings: &'a ReadStorage<'a, Facing>,

    pub battlefield: &'a Battlefield,
    pub objectives: &'a Objectives,
    pub entities: &'a Entities<'a>,

    pub visible_enemies: &'a Vec<Entity>,
}

pub trait Consideration: Send + Sync {
    fn evaluate(&self, context: &ActionContext) -> f32;
    fn name(&self) -> &str;
}

pub struct DistanceToTargetConsideration {
    curve: ResponseCurve,
}

impl DistanceToTargetConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for DistanceToTargetConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        let actor_pos = match context.positions.get(context.actor_entity) {
            Some(pos) => pos.as_battlefield_pos(),
            None => return 0.0,
        };

        let target_pos = if let Some(target_entity) = context.target_entity {
            match context.positions.get(target_entity) {
                Some(pos) => pos.as_battlefield_pos(),
                None => return 0.0,
            }
        } else if let Some(pos) = &context.target_position {
            pos
        } else {
            return 0.0;
        };

        let distance = actor_pos.distance_to(target_pos);

        let max_vision_range = context
            .visions
            .get(context.actor_entity)
            .map(|v| v.range as f32)
            .unwrap_or(10.0);

        let normalized_distance = (distance / max_vision_range).clamp(0.0, 1.0);

        self.curve.evaluate(normalized_distance)
    }

    fn name(&self) -> &str {
        "DistanceToTarget"
    }
}

pub struct AmmoLevelConsideration {
    curve: ResponseCurve,
}

impl AmmoLevelConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for AmmoLevelConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        let weapon = match context.weapons.get(context.actor_entity) {
            Some(w) => w,
            None => return 0.0,
        };

        if weapon.ammo.max_capacity == 0 {
            return 0.0;
        }

        let ammo_ratio = weapon.ammo.current as f32 / weapon.ammo.max_capacity as f32;

        self.curve.evaluate(ammo_ratio)
    }

    fn name(&self) -> &str {
        "AmmoLevel"
    }
}

pub struct HealthLevelConsideration {
    curve: ResponseCurve,
}

impl HealthLevelConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for HealthLevelConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        let health = match context.healths.get(context.actor_entity) {
            Some(h) => h,
            None => return 0.0,
        };

        let health_ratio = health.percentage();

        self.curve.evaluate(health_ratio)
    }

    fn name(&self) -> &str {
        "HealthLevel"
    }
}

pub struct HasLineOfSightConsideration {
    curve: ResponseCurve,
}

impl HasLineOfSightConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for HasLineOfSightConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        // Quick path: if the target entity is in our visible_enemies list, we have LOS
        if let Some(target_entity) = context.target_entity {
            if context.visible_enemies.contains(&target_entity) {
                return self.curve.evaluate(1.0);
            }
        }

        // Fallback: calculate FOV (for non-enemy targets or when visible_enemies is empty)
        let actor_pos = match context.positions.get(context.actor_entity) {
            Some(pos) => pos.as_battlefield_pos(),
            None => return 0.0,
        };

        let target_pos = if let Some(target_entity) = context.target_entity {
            match context.positions.get(target_entity) {
                Some(pos) => pos.as_battlefield_pos(),
                None => return 0.0,
            }
        } else if let Some(pos) = &context.target_position {
            pos
        } else {
            return 0.0;
        };

        let vision_range = context
            .visions
            .get(context.actor_entity)
            .map(|v| v.range)
            .unwrap_or(10);

        let visible_tiles = calculate_fov(actor_pos, vision_range, context.battlefield);

        let has_los = if visible_tiles.contains(target_pos) {
            1.0
        } else {
            0.0
        };

        self.curve.evaluate(has_los)
    }

    fn name(&self) -> &str {
        "HasLineOfSight"
    }
}

pub struct ThreatLevelConsideration {
    curve: ResponseCurve,
}

impl ThreatLevelConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for ThreatLevelConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        let target_entity = match context.target_entity {
            Some(e) => e,
            None => return 0.0,
        };

        let target_health = context.healths.get(target_entity);
        let target_weapon = context.weapons.get(target_entity);

        let actor_pos = match context.positions.get(context.actor_entity) {
            Some(pos) => pos.as_battlefield_pos(),
            None => return 0.0,
        };

        let target_pos = match context.positions.get(target_entity) {
            Some(pos) => pos.as_battlefield_pos(),
            None => return 0.0,
        };

        let distance = actor_pos.distance_to(target_pos);

        let mut threat = 0.0;

        if let Some(weapon) = target_weapon {
            let weapon_threat = weapon.stats.damage as f32 / 30.0;
            let range_threat = if distance <= weapon.stats.effective_range as f32 {
                1.0
            } else if distance <= weapon.stats.max_range as f32 {
                0.5
            } else {
                0.1
            };

            threat += weapon_threat * range_threat * 0.5;
        }

        if let Some(health) = target_health {
            let health_threat = health.percentage();
            threat += health_threat * 0.3;
        }

        let distance_threat = if distance < 5.0 {
            1.0
        } else if distance < 15.0 {
            0.5 + (15.0 - distance) / 20.0
        } else {
            0.1
        };
        threat += distance_threat * 0.2;

        let normalized_threat = threat.clamp(0.0, 1.0);

        self.curve.evaluate(normalized_threat)
    }

    fn name(&self) -> &str {
        "ThreatLevel"
    }
}

pub struct CoverQualityConsideration {
    curve: ResponseCurve,
}

impl CoverQualityConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for CoverQualityConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        let target_pos = if let Some(pos) = &context.target_position {
            pos
        } else if let Some(target_entity) = context.target_entity {
            match context.positions.get(target_entity) {
                Some(pos) => pos.as_battlefield_pos(),
                None => return 0.0,
            }
        } else {
            return 0.0;
        };

        let cover_bonus = match context.battlefield.get_tile(target_pos) {
            Some(tile) => tile.terrain.cover_bonus(),
            None => 0.0,
        };

        let max_cover_bonus = 0.5;
        let normalized_cover = (cover_bonus / max_cover_bonus).clamp(0.0, 1.0);

        self.curve.evaluate(normalized_cover)
    }

    fn name(&self) -> &str {
        "CoverQuality"
    }
}

pub struct ObjectiveProximityConsideration {
    curve: ResponseCurve,
}

impl ObjectiveProximityConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for ObjectiveProximityConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        let actor_pos = match context.positions.get(context.actor_entity) {
            Some(pos) => pos.as_battlefield_pos(),
            None => return 0.0,
        };

        let actor_faction = match context.soldiers.get(context.actor_entity) {
            Some(s) => s.faction,
            None => return 0.0,
        };

        let objective_pos = match context.objectives.get_enemy_flag_position(actor_faction) {
            Some(pos) => pos,
            None => return 0.0,
        };

        let distance = actor_pos.distance_to(&objective_pos);

        let battlefield_size = (context.battlefield.width().pow(2) + context.battlefield.height().pow(2)) as f32;
        let max_distance = battlefield_size.sqrt();

        let normalized_distance = (distance / max_distance).clamp(0.0, 1.0);

        self.curve.evaluate(normalized_distance)
    }

    fn name(&self) -> &str {
        "ObjectiveProximity"
    }
}

pub struct AlliesNearbyConsideration {
    curve: ResponseCurve,
}

impl AlliesNearbyConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for AlliesNearbyConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        let actor_pos = match context.positions.get(context.actor_entity) {
            Some(pos) => pos.as_battlefield_pos(),
            None => return 0.0,
        };

        let actor_faction = match context.soldiers.get(context.actor_entity) {
            Some(s) => s.faction,
            None => return 0.0,
        };

        let vision_range = context
            .visions
            .get(context.actor_entity)
            .map(|v| v.range as f32)
            .unwrap_or(10.0);

        let mut ally_count = 0;

        for (entity, pos, soldier) in (context.entities, context.positions, context.soldiers).join() {
            if entity == context.actor_entity {
                continue;
            }

            if soldier.faction != actor_faction {
                continue;
            }

            let distance = actor_pos.distance_to(pos.as_battlefield_pos());
            if distance <= vision_range {
                ally_count += 1;
            }
        }

        let max_allies = 5;
        let normalized_allies = (ally_count as f32 / max_allies as f32).clamp(0.0, 1.0);

        self.curve.evaluate(normalized_allies)
    }

    fn name(&self) -> &str {
        "AlliesNearby"
    }
}

pub struct NearbyOfficerConsideration {
    curve: ResponseCurve,
}

impl NearbyOfficerConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for NearbyOfficerConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        let actor_rank = context
            .soldiers
            .get(context.actor_entity)
            .map(|s| s.rank)
            .unwrap_or(Rank::Private);

        if matches!(
            actor_rank,
            Rank::Lieutenant | Rank::Captain | Rank::Sergeant
        ) {
            return 1.0;
        }

        let actor_pos = match context.positions.get(context.actor_entity) {
            Some(pos) => pos.as_battlefield_pos(),
            None => return 0.5,
        };

        let actor_faction = match context.soldiers.get(context.actor_entity) {
            Some(s) => s.faction,
            None => return 0.5,
        };

        let mut nearest_officer_dist = f32::MAX;

        for (_entity, soldier, pos) in (context.entities, context.soldiers, context.positions).join()
        {
            if soldier.faction != actor_faction {
                continue;
            }

            if matches!(soldier.rank, Rank::Lieutenant | Rank::Captain) {
                let distance = actor_pos.distance_to(pos.as_battlefield_pos());
                nearest_officer_dist = nearest_officer_dist.min(distance);
            }
        }

        if nearest_officer_dist == f32::MAX {
            return 0.5;
        }

        let normalized_dist = (nearest_officer_dist / 20.0).min(1.0);

        self.curve.evaluate(normalized_dist)
    }

    fn name(&self) -> &str {
        "NearbyOfficer"
    }
}
