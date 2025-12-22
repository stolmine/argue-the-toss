use crate::ai::response_curves::ResponseCurve;
use crate::components::{
    facing::Facing, health::Health, position::Position, soldier::{Faction, Rank, Soldier},
    vision::Vision, weapon::Weapon,
};
use crate::game_logic::battlefield::{Battlefield, Position as BattlefieldPos};
use crate::game_logic::line_of_sight::calculate_fov;
use crate::game_logic::objectives::Objectives;
use specs::{Entities, Entity, Join, ReadStorage};
use std::time::Instant;

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

    // Wrapper with timing for debugging
    fn evaluate_with_timing(&self, context: &ActionContext) -> f32 {
        if cfg!(debug_assertions) {
            let start = Instant::now();
            let result = self.evaluate(context);
            let elapsed = start.elapsed();

            if elapsed.as_micros() > 100 {  // Log if > 100μs
                eprintln!("[PERF] {} took {}μs", self.name(), elapsed.as_micros());
            }
            result
        } else {
            self.evaluate(context)
        }
    }
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

// ============================================================================
// Tactical Movement Considerations
// ============================================================================

/// Evaluates danger from being exposed (current cover vs enemy line of sight)
pub struct ExposedDangerConsideration {
    curve: ResponseCurve,
}

impl ExposedDangerConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for ExposedDangerConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        let actor_pos = match context.positions.get(context.actor_entity) {
            Some(pos) => pos.as_battlefield_pos(),
            None => return 0.0,
        };

        // Get current cover quality
        let current_cover = context.battlefield
            .get_tile(actor_pos)
            .map(|t| t.terrain.cover_bonus() as f32)
            .unwrap_or(0.0);

        // Count enemies that can see us
        let enemies_with_los = context.visible_enemies.len() as f32;

        // High danger if: low cover + many enemies can see us
        let cover_factor = 1.0 - (current_cover as f32 / 100.0).min(1.0);
        let exposure_factor = (enemies_with_los / 5.0).min(1.0); // Normalize to ~5 enemies

        let danger = (cover_factor * 0.6) + (exposure_factor * 0.4);

        self.curve.evaluate(danger.clamp(0.0, 1.0))
    }

    fn name(&self) -> &str {
        "ExposedDanger"
    }
}

/// Evaluates tactical advantage of target position (range, elevation, flanking)
pub struct TacticalAdvantageConsideration {
    curve: ResponseCurve,
}

impl TacticalAdvantageConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for TacticalAdvantageConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        let actor_pos = match context.positions.get(context.actor_entity) {
            Some(pos) => pos.as_battlefield_pos(),
            None => return 0.0,
        };

        let target_pos = match &context.target_position {
            Some(pos) => pos,
            None => return 0.0,
        };

        // Get target cover quality (moving TO better cover is advantageous)
        let target_cover = context.battlefield
            .get_tile(target_pos)
            .map(|t| t.terrain.cover_bonus() as f32)
            .unwrap_or(0.0);

        let current_cover = context.battlefield
            .get_tile(actor_pos)
            .map(|t| t.terrain.cover_bonus() as f32)
            .unwrap_or(0.0);

        // Cover improvement factor
        let cover_improvement = ((target_cover - current_cover) / 100.0).max(0.0).min(1.0);

        // Range factor - check if we'll be at better weapon range
        let weapon = context.weapons.get(context.actor_entity);
        let range_advantage = if let Some(weapon) = weapon {
            if let Some(&first_enemy) = context.visible_enemies.first() {
                if let Some(enemy_pos) = context.positions.get(first_enemy) {
                    let current_dist = actor_pos.distance_to(enemy_pos.as_battlefield_pos());
                    let target_dist = target_pos.distance_to(enemy_pos.as_battlefield_pos());
                    let effective_range = weapon.stats.effective_range as f32;

                    // Prefer moving toward effective range
                    let current_range_quality = 1.0 - ((current_dist - effective_range).abs() / effective_range).min(1.0);
                    let target_range_quality = 1.0 - ((target_dist - effective_range).abs() / effective_range).min(1.0);

                    (target_range_quality - current_range_quality).max(0.0)
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Combine factors
        let advantage = (cover_improvement * 0.7) + (range_advantage * 0.3);

        self.curve.evaluate(advantage.clamp(0.0, 1.0))
    }

    fn name(&self) -> &str {
        "TacticalAdvantage"
    }
}

/// Evaluates local force balance (friendly:enemy ratio)
/// OPTIMIZED: Uses visible_enemies heuristic instead of entity iteration
pub struct ForceBalanceConsideration {
    curve: ResponseCurve,
}

impl ForceBalanceConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for ForceBalanceConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        // Fast heuristic: use visible_enemies count as threat indicator
        // Assumes roughly balanced forces (good enough for tactical decisions)
        let nearby_enemies = context.visible_enemies.len();

        if nearby_enemies == 0 {
            return self.curve.evaluate(0.0);
        }

        // Heuristic: more enemies = higher imbalance/threat
        // 1-2 enemies = manageable (0.2-0.3)
        // 3-4 enemies = concerning (0.5-0.6)
        // 5-6 enemies = critical (0.8-1.0)
        let imbalance = (nearby_enemies as f32 / 6.0).min(1.0);

        self.curve.evaluate(imbalance)
    }

    fn name(&self) -> &str {
        "ForceBalance"
    }
}

/// Evaluates proximity to friendly support
/// OPTIMIZED: Simple heuristic instead of iterating all entities
pub struct SupportProximityConsideration {
    curve: ResponseCurve,
}

impl SupportProximityConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for SupportProximityConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        // Fast heuristic: if enemies are visible but in small numbers,
        // assume friendlies are nearby (shared vision system)
        // If many enemies visible, assume we're isolated/forward

        let enemy_count = context.visible_enemies.len();

        if enemy_count == 0 {
            // No enemies = probably with friendlies or safe area
            return self.curve.evaluate(0.0);
        }

        // Simple model:
        // 1-2 enemies visible = probably in supported position (0.2)
        // 3-5 enemies visible = might be isolated (0.5-0.7)
        // 6+ enemies visible = likely isolated/forward (1.0)
        let isolation = (enemy_count as f32 / 6.0).min(1.0);

        self.curve.evaluate(isolation)
    }

    fn name(&self) -> &str {
        "SupportProximity"
    }
}

/// Evaluates need to move for objectives
pub struct ObjectivePressureConsideration {
    curve: ResponseCurve,
}

impl ObjectivePressureConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for ObjectivePressureConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        let actor_pos = match context.positions.get(context.actor_entity) {
            Some(pos) => pos.as_battlefield_pos(),
            None => return 0.0,
        };

        let actor_faction = match context.soldiers.get(context.actor_entity) {
            Some(soldier) => soldier.faction,
            None => return 0.0,
        };

        let target_pos = match &context.target_position {
            Some(pos) => pos,
            None => return 0.0,
        };

        // Find nearest enemy-controlled objective
        let mut nearest_obj_dist = f32::MAX;
        let mut objective_found = false;

        for flag in context.objectives.flags.values() {
            if flag.owning_faction != actor_faction {
                let dist = actor_pos.distance_to(&flag.position);
                if dist < nearest_obj_dist {
                    nearest_obj_dist = dist;
                    objective_found = true;
                }
            }
        }

        if !objective_found {
            return self.curve.evaluate(0.0);
        }

        // Calculate if moving toward objective
        let current_dist = nearest_obj_dist;
        let target_obj_dist = context.objectives.flags.values()
            .filter(|f| f.owning_faction != actor_faction)
            .map(|f| target_pos.distance_to(&f.position))
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(f32::MAX);

        // Moving toward objective = high value
        // Moving away = low value
        if target_obj_dist < current_dist {
            let improvement = (current_dist - target_obj_dist) / current_dist;
            self.curve.evaluate(improvement.clamp(0.0, 1.0))
        } else {
            self.curve.evaluate(0.0)
        }
    }

    fn name(&self) -> &str {
        "ObjectivePressure"
    }
}

/// Evaluates need to retreat based on health and ammo
pub struct RetreatNecessityConsideration {
    curve: ResponseCurve,
}

impl RetreatNecessityConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for RetreatNecessityConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        let health = match context.healths.get(context.actor_entity) {
            Some(h) => h,
            None => return 0.0,
        };

        let weapon = context.weapons.get(context.actor_entity);

        // Health factor (low health = high retreat need)
        let health_factor = 1.0 - health.percentage();

        // Ammo factor (low ammo = moderate retreat need)
        let ammo_factor = if let Some(weapon) = weapon {
            let ammo_ratio = weapon.ammo.current as f32 / weapon.ammo.max_capacity as f32;
            (1.0 - ammo_ratio) * 0.5 // Less weight than health
        } else {
            0.0
        };

        // Enemy presence factor (wounded + enemies nearby = urgent retreat)
        let enemy_pressure = if !context.visible_enemies.is_empty() && health_factor > 0.3 {
            0.3 // Boost retreat urgency if wounded and enemies visible
        } else {
            0.0
        };

        let retreat_necessity = health_factor + ammo_factor + enemy_pressure;

        self.curve.evaluate(retreat_necessity.clamp(0.0, 1.0))
    }

    fn name(&self) -> &str {
        "RetreatNecessity"
    }
}

/// Evaluates whether enemies are visible (for aggressive seeking)
pub struct NoEnemiesVisibleConsideration {
    curve: ResponseCurve,
}

impl NoEnemiesVisibleConsideration {
    pub fn new(curve: ResponseCurve) -> Self {
        Self { curve }
    }
}

impl Consideration for NoEnemiesVisibleConsideration {
    fn evaluate(&self, context: &ActionContext) -> f32 {
        // Returns 1.0 if no enemies visible, 0.0 if enemies visible
        let no_enemies = if context.visible_enemies.is_empty() {
            1.0
        } else {
            0.0
        };

        self.curve.evaluate(no_enemies)
    }

    fn name(&self) -> &str {
        "NoEnemiesVisible"
    }
}
