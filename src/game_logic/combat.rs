// Combat calculation logic
// Hitscan combat system with range-based accuracy

use crate::components::{health::Health, position::Position, weapon::Weapon};
use crate::game_logic::battlefield::{Battlefield, Position as BattlefieldPos};
use crate::game_logic::line_of_sight::calculate_fov;
use rand::Rng;

/// Result of a combat calculation
#[derive(Debug, Clone)]
pub struct CombatResult {
    pub hit: bool,
    pub damage: i32,
    pub hit_chance: f32,
    pub distance: i32,
    pub blocked_by_los: bool,
    pub cover_bonus: f32,
}

/// Calculate if a shot hits and how much damage it deals
///
/// # Parameters
/// - `weapon`: The weapon being used
/// - `shooter_pos`: Position of the shooter
/// - `target_pos`: Position of the target
/// - `battlefield`: The battlefield (for LOS checks)
/// - `shooter_vision`: Vision range of shooter (for LOS calculation)
///
/// # Returns
/// CombatResult with hit/miss, damage, and other details
pub fn calculate_shot(
    weapon: &Weapon,
    shooter_pos: &Position,
    target_pos: &Position,
    battlefield: &Battlefield,
    shooter_vision: i32,
) -> CombatResult {
    // Calculate distance to target
    let distance = calculate_distance(shooter_pos, target_pos);

    // Check if target is in range
    if distance > weapon.stats.max_range {
        return CombatResult {
            hit: false,
            damage: 0,
            hit_chance: 0.0,
            distance,
            blocked_by_los: false,
            cover_bonus: 0.0,
        };
    }

    // Check line of sight
    let has_los = check_line_of_sight(shooter_pos, target_pos, battlefield, shooter_vision);
    if !has_los {
        return CombatResult {
            hit: false,
            damage: 0,
            hit_chance: 0.0,
            distance,
            blocked_by_los: true,
            cover_bonus: 0.0,
        };
    }

    // Get target's cover bonus from terrain
    let target_battlefield_pos = BattlefieldPos::new(target_pos.x(), target_pos.y());
    let cover_bonus = battlefield
        .get_tile(&target_battlefield_pos)
        .map(|tile| tile.terrain.cover_bonus())
        .unwrap_or(0.0);

    // Calculate hit chance based on range
    let hit_chance = calculate_hit_chance(weapon, distance);

    // Roll to hit
    let mut rng = rand::rng();
    let roll: f32 = rng.random();
    let hit = roll < hit_chance;

    let damage = if hit {
        // Apply cover damage reduction
        let base_damage = weapon.stats.damage as f32;
        let reduced_damage = base_damage * (1.0 - cover_bonus);
        reduced_damage.round() as i32
    } else {
        0
    };

    CombatResult {
        hit,
        damage,
        hit_chance,
        distance,
        blocked_by_los: false,
        cover_bonus,
    }
}

/// Calculate hit chance based on weapon and distance
///
/// Accuracy degrades linearly from effective_range to max_range:
/// - At 0 distance: base_accuracy
/// - At effective_range: base_accuracy
/// - At max_range: base_accuracy * 0.3
/// - Beyond max_range: 0.0
fn calculate_hit_chance(weapon: &Weapon, distance: i32) -> f32 {
    if distance <= weapon.stats.effective_range {
        // Within effective range: full accuracy
        weapon.stats.base_accuracy
    } else if distance <= weapon.stats.max_range {
        // Beyond effective range: linear degradation
        let range_beyond_effective = distance - weapon.stats.effective_range;
        let total_degradation_range = weapon.stats.max_range - weapon.stats.effective_range;
        let degradation_factor = range_beyond_effective as f32 / total_degradation_range as f32;

        // Degrade from base_accuracy to base_accuracy * 0.3
        let min_accuracy = weapon.stats.base_accuracy * 0.3;
        let accuracy_drop = weapon.stats.base_accuracy - min_accuracy;

        weapon.stats.base_accuracy - (accuracy_drop * degradation_factor)
    } else {
        // Out of range
        0.0
    }
}

/// Calculate distance between two positions (Euclidean distance, rounded up)
fn calculate_distance(pos1: &Position, pos2: &Position) -> i32 {
    let dx = (pos1.x() - pos2.x()) as f32;
    let dy = (pos1.y() - pos2.y()) as f32;
    (dx * dx + dy * dy).sqrt().ceil() as i32
}

/// Check if shooter has line of sight to target
/// Uses the FOV system to determine visibility
fn check_line_of_sight(
    shooter_pos: &Position,
    target_pos: &Position,
    battlefield: &Battlefield,
    vision_range: i32,
) -> bool {
    // Calculate FOV from shooter position
    let shooter_battlefield_pos = BattlefieldPos::new(shooter_pos.x(), shooter_pos.y());
    let visible_tiles = calculate_fov(&shooter_battlefield_pos, vision_range, battlefield);

    // Check if target position is in visible tiles
    let target_battlefield_pos = BattlefieldPos::new(target_pos.x(), target_pos.y());
    visible_tiles.contains(&target_battlefield_pos)
}

/// Apply damage to a health component
/// Returns true if the entity is still alive
pub fn apply_damage(health: &mut Health, damage: i32) -> bool {
    health.take_damage(damage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_calculation() {
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 4);
        assert_eq!(calculate_distance(&pos1, &pos2), 5); // 3-4-5 triangle
    }

    #[test]
    fn test_hit_chance_within_effective_range() {
        let weapon = Weapon::rifle();
        let distance = 10; // Within effective range (15)
        let hit_chance = calculate_hit_chance(&weapon, distance);
        assert_eq!(hit_chance, weapon.stats.base_accuracy);
    }

    #[test]
    fn test_hit_chance_beyond_effective_range() {
        let weapon = Weapon::rifle();
        let distance = 30; // At max range
        let hit_chance = calculate_hit_chance(&weapon, distance);
        assert!(hit_chance < weapon.stats.base_accuracy);
        assert!(hit_chance >= weapon.stats.base_accuracy * 0.3);
    }

    #[test]
    fn test_hit_chance_out_of_range() {
        let weapon = Weapon::rifle();
        let distance = 35; // Beyond max range (30)
        let hit_chance = calculate_hit_chance(&weapon, distance);
        assert_eq!(hit_chance, 0.0);
    }

    #[test]
    fn test_random_distribution() {
        // Test that the RNG is actually producing values in [0.0, 1.0)
        let mut rng = rand::rng();
        for _ in 0..100 {
            let roll: f32 = rng.random();
            assert!(roll >= 0.0 && roll < 1.0, "Random value out of range: {}", roll);
        }
    }

    #[test]
    fn test_multiple_shots() {
        // Test that shots actually hit sometimes at close range
        let weapon = Weapon::rifle();
        let shooter_pos = Position::new(50, 50);
        let target_pos = Position::new(55, 52); // ~7 tiles away
        let battlefield = Battlefield::new(100, 100);
        let shooter_vision = 10;

        let mut hits = 0;
        for _ in 0..100 {
            let result = calculate_shot(&weapon, &shooter_pos, &target_pos, &battlefield, shooter_vision);
            if result.hit {
                hits += 1;
            }
        }

        // At ~7 tiles distance (within effective range), we should have base_accuracy (70%)
        // Over 100 shots, we expect around 70 hits +/- some variance
        // Allow for statistical variance: 50-90 hits is reasonable
        println!("Hits: {}/100 ({}%)", hits, hits);
        assert!(hits >= 50 && hits <= 90, "Hit rate seems wrong: {}/100. Expected around 70/100", hits);
    }
}
