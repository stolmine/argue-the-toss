// Vision Cone System
// Implements directional vision with main cone (120°) and peripheral vision (60° each side)

use crate::components::facing::Direction8;
use crate::game_logic::battlefield::{Battlefield, Position};
use crate::game_logic::line_of_sight::calculate_fov;
use std::collections::HashSet;

/// Vision level for a tile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityLevel {
    /// Not visible (behind entity, outside cone, or blocked by LOS)
    Hidden,
    /// Main vision cone (120° arc, full brightness)
    MainVision,
    /// Peripheral vision (60° each side, dimmed)
    Peripheral,
}

/// Calculate vision cone from a position with facing direction
/// Returns two sets: main vision tiles and peripheral vision tiles
pub fn calculate_vision_cone(
    origin: &Position,
    facing: Direction8,
    vision_range: i32,
    battlefield: &Battlefield,
) -> (HashSet<Position>, HashSet<Position>) {
    // First get all visible tiles using existing LOS system
    let all_visible = calculate_fov(origin, vision_range, battlefield);

    // Get facing angle
    let facing_angle = facing.angle_degrees();

    let mut main_vision = HashSet::new();
    let mut peripheral_vision = HashSet::new();

    // Filter tiles by angle from facing direction
    for pos in all_visible {
        // Calculate angle from origin to this position
        let dx = (pos.x - origin.x) as f32;
        let dy = (pos.y - origin.y) as f32;

        // Skip origin tile (always visible)
        if dx.abs() < 0.1 && dy.abs() < 0.1 {
            main_vision.insert(pos);
            continue;
        }

        // Calculate angle to target (atan2 returns -π to π, we need 0-360)
        // Note: Screen coords have Y increasing downward (0,0 at top-left)
        // For our facing system: North=0°, East=90°, South=180°, West=270° (clockwise)
        // We need to negate dy because North is negative Y direction in screen coords
        let angle_rad = (-dy).atan2(dx);
        let mut angle_deg = angle_rad.to_degrees();

        // Convert from mathematical angle (East=0°, counter-clockwise)
        // to our facing system (North=0°, clockwise)
        angle_deg = 90.0 - angle_deg;

        // Normalize to 0-360
        while angle_deg < 0.0 {
            angle_deg += 360.0;
        }
        while angle_deg >= 360.0 {
            angle_deg -= 360.0;
        }

        // Calculate angular difference from facing direction
        let mut angle_diff = (angle_deg - facing_angle).abs();

        // Handle wraparound (e.g., facing N=0° and looking at NW=315°)
        if angle_diff > 180.0 {
            angle_diff = 360.0 - angle_diff;
        }

        // Categorize based on angle difference
        if angle_diff <= 60.0 {
            // Within ±60° = main cone (120° total)
            main_vision.insert(pos);
        } else if angle_diff <= 90.0 {
            // Within ±60° to ±90° = peripheral (60° each side)
            peripheral_vision.insert(pos);
        }
        // Else: angle_diff > 90° = behind, not visible
    }

    (main_vision, peripheral_vision)
}

/// Get visibility level for a specific tile
pub fn get_visibility_level(
    origin: &Position,
    target: &Position,
    facing: Direction8,
    vision_range: i32,
    battlefield: &Battlefield,
) -> VisibilityLevel {
    // Calculate vision cones
    let (main_vision, peripheral_vision) = calculate_vision_cone(origin, facing, vision_range, battlefield);

    if main_vision.contains(target) {
        VisibilityLevel::MainVision
    } else if peripheral_vision.contains(target) {
        VisibilityLevel::Peripheral
    } else {
        VisibilityLevel::Hidden
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_logic::battlefield::TerrainType;

    #[test]
    fn test_vision_cone_north() {
        let mut bf = Battlefield::new(20, 20);
        // Fill with no-mans-land (doesn't block vision)
        for x in 0..20 {
            for y in 0..20 {
                bf.set_terrain(Position::new(x, y), TerrainType::NoMansLand);
            }
        }

        let origin = Position::new(10, 10);
        let facing = Direction8::N;
        let range = 5;

        let (main, peripheral) = calculate_vision_cone(&origin, facing, range, &bf);

        // Tile directly north should be in main vision
        assert!(main.contains(&Position::new(10, 8)));

        // Tiles to the sides at ±60° should be in main vision
        assert!(main.contains(&Position::new(12, 8))); // NE-ish
        assert!(main.contains(&Position::new(8, 8)));  // NW-ish

        // Tiles directly east/west should be in peripheral or not visible
        // (depends on exact angle, but should NOT be in main vision)
        assert!(!main.contains(&Position::new(15, 10))); // Due East
    }

    #[test]
    fn test_vision_cone_east() {
        let mut bf = Battlefield::new(20, 20);
        for x in 0..20 {
            for y in 0..20 {
                bf.set_terrain(Position::new(x, y), TerrainType::NoMansLand);
            }
        }

        let origin = Position::new(10, 10);
        let facing = Direction8::E;
        let range = 5;

        let (main, _peripheral) = calculate_vision_cone(&origin, facing, range, &bf);

        // Tile directly east should be in main vision
        assert!(main.contains(&Position::new(12, 10)));

        // Tile directly west should NOT be in main vision (behind)
        assert!(!main.contains(&Position::new(8, 10)));
    }

    #[test]
    fn test_vision_cone_south() {
        let mut bf = Battlefield::new(20, 20);
        for x in 0..20 {
            for y in 0..20 {
                bf.set_terrain(Position::new(x, y), TerrainType::NoMansLand);
            }
        }

        let origin = Position::new(10, 10);
        let facing = Direction8::S;
        let range = 5;

        let (main, _peripheral) = calculate_vision_cone(&origin, facing, range, &bf);

        // Tile directly south (positive Y) should be in main vision
        assert!(main.contains(&Position::new(10, 12)));

        // Tile directly north (negative Y) should NOT be in main vision (behind)
        assert!(!main.contains(&Position::new(10, 8)));
    }

    #[test]
    fn test_vision_cone_west() {
        let mut bf = Battlefield::new(20, 20);
        for x in 0..20 {
            for y in 0..20 {
                bf.set_terrain(Position::new(x, y), TerrainType::NoMansLand);
            }
        }

        let origin = Position::new(10, 10);
        let facing = Direction8::W;
        let range = 5;

        let (main, _peripheral) = calculate_vision_cone(&origin, facing, range, &bf);

        // Tile directly west should be in main vision
        assert!(main.contains(&Position::new(8, 10)));

        // Tile directly east should NOT be in main vision (behind)
        assert!(!main.contains(&Position::new(12, 10)));
    }
}
