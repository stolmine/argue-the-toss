// Line-of-sight and field-of-view calculations

use crate::game_logic::battlefield::{Battlefield, Position};
use bracket_lib::prelude::*;
use std::collections::HashSet;

/// Calculate field of view from a position with given range
pub fn calculate_fov(
    origin: &Position,
    range: i32,
    battlefield: &Battlefield,
) -> HashSet<Position> {
    let mut visible_tiles = HashSet::new();

    // bracket-lib FOV requires a map that implements Algorithm2D and BaseMap
    // We'll create a wrapper for our Battlefield
    let map = BattlefieldFOVMap::new(battlefield);

    // Use bracket-lib's field_of_view_set
    // This uses symmetric shadowcasting algorithm
    let visible = field_of_view_set(Point::new(origin.x, origin.y), range, &map);

    // Convert bracket-lib Points back to our Position type
    for point in visible {
        visible_tiles.insert(Position::new(point.x, point.y));
    }

    visible_tiles
}

/// Wrapper to make Battlefield compatible with bracket-lib FOV
struct BattlefieldFOVMap<'a> {
    battlefield: &'a Battlefield,
}

impl<'a> BattlefieldFOVMap<'a> {
    fn new(battlefield: &'a Battlefield) -> Self {
        Self { battlefield }
    }
}

impl<'a> Algorithm2D for BattlefieldFOVMap<'a> {
    fn dimensions(&self) -> Point {
        Point::new(
            self.battlefield.width() as i32,
            self.battlefield.height() as i32,
        )
    }

    fn index_to_point2d(&self, idx: usize) -> Point {
        let width = self.battlefield.width() as i32;
        Point::new(idx as i32 % width, idx as i32 / width)
    }
}

impl<'a> BaseMap for BattlefieldFOVMap<'a> {
    fn is_opaque(&self, idx: usize) -> bool {
        let point = self.index_to_point2d(idx);
        let pos = Position::new(point.x, point.y);

        if let Some(tile) = self.battlefield.get_tile(&pos) {
            tile.terrain.blocks_los()
        } else {
            false // Out of bounds = not opaque
        }
    }

    fn get_available_exits(&self, _idx: usize) -> SmallVec<[(usize, f32); 10]> {
        // Not needed for FOV, only for pathfinding
        SmallVec::new()
    }
}
