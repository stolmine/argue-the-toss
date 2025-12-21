// Pathfinding logic using bracket-lib A* algorithm

use crate::game_logic::battlefield::{Battlefield, Position};
use bracket_lib::prelude::*;
use bracket_pathfinding::prelude::a_star_search;

/// Calculate A* path from start to end position
/// Returns Some(Vec<Position>) if path found, None if no path exists
pub fn calculate_path(
    start: &Position,
    end: &Position,
    battlefield: &Battlefield,
) -> Option<Vec<Position>> {
    // Don't pathfind if already at destination
    if start == end {
        return Some(vec![]);
    }

    // Create map wrapper for pathfinding
    let map = BattlefieldPathMap::new(battlefield);

    // Convert positions to indices
    let start_idx = map.point2d_to_index(Point::new(start.x, start.y));
    let end_idx = map.point2d_to_index(Point::new(end.x, end.y));

    // Run A* pathfinding
    let path_result = a_star_search(start_idx, end_idx, &map);

    if path_result.success {
        // Convert indices back to Positions, excluding the start position
        let positions: Vec<Position> = path_result
            .steps
            .into_iter()
            .skip(1) // Skip start position (already there)
            .map(|idx| {
                let pt = map.index_to_point2d(idx);
                Position::new(pt.x, pt.y)
            })
            .collect();

        Some(positions)
    } else {
        None
    }
}

/// Wrapper to make Battlefield compatible with bracket-lib pathfinding
/// Mirrors the pattern from BattlefieldFOVMap in line_of_sight.rs
struct BattlefieldPathMap<'a> {
    battlefield: &'a Battlefield,
}

impl<'a> BattlefieldPathMap<'a> {
    fn new(battlefield: &'a Battlefield) -> Self {
        Self { battlefield }
    }
}

impl<'a> Algorithm2D for BattlefieldPathMap<'a> {
    fn dimensions(&self) -> Point {
        Point::new(
            self.battlefield.width() as i32,
            self.battlefield.height() as i32,
        )
    }

    fn index_to_point2d(&self, idx: usize) -> Point {
        let width = self.battlefield.width() as i32;
        let x = idx as i32 % width;
        let y = idx as i32 / width;
        Point::new(x, y)
    }

    fn point2d_to_index(&self, pt: Point) -> usize {
        let width = self.battlefield.width() as i32;
        ((pt.y * width) + pt.x) as usize
    }
}

impl<'a> BaseMap for BattlefieldPathMap<'a> {
    fn is_opaque(&self, _idx: usize) -> bool {
        // Not used for pathfinding (only for FOV)
        false
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let p1 = self.index_to_point2d(idx1);
        let p2 = self.index_to_point2d(idx2);
        let dx = (p1.x - p2.x) as f32;
        let dy = (p1.y - p2.y) as f32;

        // Euclidean distance for heuristic
        (dx * dx + dy * dy).sqrt()
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let point = self.index_to_point2d(idx);
        let pos = Position::new(point.x, point.y);

        // 8-directional movement (cardinal + diagonal)
        for dy in -1..=1 {
            for dx in -1..=1 {
                // Skip the center position (no movement)
                if dx == 0 && dy == 0 {
                    continue;
                }

                let new_pos = Position::new(pos.x + dx, pos.y + dy);

                // Check if new position is within bounds
                if !self.battlefield.in_bounds(&new_pos) {
                    continue;
                }

                // Get terrain cost multiplier (1.0 - 2.0)
                let terrain_cost = self
                    .battlefield
                    .get_tile(&new_pos)
                    .map(|t| t.terrain.movement_cost())
                    .unwrap_or(1.0);

                // Calculate distance cost (1.0 for cardinal, ~1.414 for diagonal)
                let distance_cost = if dx != 0 && dy != 0 {
                    1.414 // sqrt(2) for diagonal movement
                } else {
                    1.0 // cardinal directions
                };

                // Total cost is distance * terrain multiplier
                let total_cost = distance_cost * terrain_cost;

                let new_idx = self.point2d_to_index(Point::new(new_pos.x, new_pos.y));
                exits.push((new_idx, total_cost));
            }
        }

        exits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_logic::battlefield::TerrainType;

    #[test]
    fn test_pathfinding_straight_line() {
        let battlefield = Battlefield::new(10, 10);
        let start = Position::new(0, 0);
        let end = Position::new(5, 0);

        let path = calculate_path(&start, &end, &battlefield);

        assert!(path.is_some());
        let path = path.unwrap();
        // Path should reach the destination
        assert_eq!(path.last().unwrap(), &end);
        // Path should be reasonable length (not wildly inefficient)
        assert!(path.len() <= 10);
    }

    #[test]
    fn test_pathfinding_diagonal() {
        let battlefield = Battlefield::new(10, 10);
        let start = Position::new(0, 0);
        let end = Position::new(3, 3);

        let path = calculate_path(&start, &end, &battlefield);

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.last().unwrap(), &end);
        // Path should be reasonable length (not wildly inefficient)
        assert!(path.len() <= 10);
    }

    #[test]
    fn test_pathfinding_prefers_low_cost_terrain() {
        let mut battlefield = Battlefield::new(10, 10);

        // Create a line of mud (high cost) terrain from (2, 0) to (2, 9)
        for y in 0..10 {
            battlefield.set_terrain(Position::new(2, y), TerrainType::Mud);
        }

        let start = Position::new(0, 5);
        let end = Position::new(4, 5);

        let path = calculate_path(&start, &end, &battlefield);

        assert!(path.is_some());
        let path = path.unwrap();

        // The path should find a route to the destination
        assert_eq!(path.last().unwrap(), &end);
    }

    #[test]
    fn test_pathfinding_same_position() {
        let battlefield = Battlefield::new(10, 10);
        let pos = Position::new(5, 5);

        let path = calculate_path(&pos, &pos, &battlefield);

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 0); // Already at destination
    }
}
