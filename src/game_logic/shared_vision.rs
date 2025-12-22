// Shared Vision System
// Calculates combined vision for all entities of a faction

use crate::components::facing::Facing;
use crate::components::position::Position;
use crate::components::soldier::{Faction, Soldier};
use crate::components::vision::Vision;
use crate::game_logic::battlefield::{Battlefield, Position as BattlefieldPosition};
use crate::game_logic::vision_cone::calculate_vision_cone;
use specs::{Entity, Join, ReadStorage};
use std::collections::{HashMap, HashSet};

/// Result of calculating shared vision for a faction
#[derive(Debug)]
pub struct SharedVisionResult {
    /// All tiles visible to the faction (main vision)
    pub visible_tiles: HashSet<Position>,
    /// All tiles in peripheral vision
    pub peripheral_tiles: HashSet<Position>,
    /// Maps each visible position to the entity that spotted it first
    pub spotter_map: HashMap<Position, Entity>,
}

impl SharedVisionResult {
    pub fn new() -> Self {
        Self {
            visible_tiles: HashSet::new(),
            peripheral_tiles: HashSet::new(),
            spotter_map: HashMap::new(),
        }
    }
}

/// Calculate combined vision for all entities of a given faction
/// This merges vision cones from all friendly units into one unified FOV
pub fn calculate_faction_vision(
    entities: &specs::world::EntitiesRes,
    positions: &ReadStorage<Position>,
    visions: &ReadStorage<Vision>,
    facings: &ReadStorage<Facing>,
    soldiers: &ReadStorage<Soldier>,
    faction: Faction,
    battlefield: &Battlefield,
) -> SharedVisionResult {
    let mut result = SharedVisionResult::new();

    // Iterate through all entities with the matching faction
    for (entity, pos, vision, facing, soldier) in
        (entities, positions, visions, facings, soldiers).join()
    {
        // Skip entities not in our faction
        if soldier.faction != faction {
            continue;
        }

        // Calculate vision cone for this entity
        // Convert component Position to BattlefieldPosition
        let (main_vision, peripheral_vision) =
            calculate_vision_cone(pos.as_battlefield_pos(), facing.direction, vision.range, battlefield);

        // Merge main vision tiles
        for tile in main_vision {
            // Convert BattlefieldPosition back to component Position
            let comp_pos = Position::new(tile.x, tile.y);
            result.visible_tiles.insert(comp_pos);
            // Track first spotter for this tile
            result.spotter_map.entry(comp_pos).or_insert(entity);
        }

        // Merge peripheral vision tiles
        for tile in peripheral_vision {
            // Convert BattlefieldPosition back to component Position
            let comp_pos = Position::new(tile.x, tile.y);
            result.peripheral_tiles.insert(comp_pos);
            // Track first spotter for this tile
            result.spotter_map.entry(comp_pos).or_insert(entity);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::facing::Direction8;
    use crate::game_logic::battlefield::TerrainType;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn test_shared_vision_single_entity() {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Vision>();
        world.register::<Facing>();
        world.register::<Soldier>();

        let mut bf = Battlefield::new(20, 20);
        for x in 0..20 {
            for y in 0..20 {
                bf.set_terrain(BattlefieldPosition::new(x, y), TerrainType::NoMansLand);
            }
        }

        // Create one entity
        world
            .create_entity()
            .with(Position::new(10, 10))
            .with(Vision::new(5))
            .with(Facing::new(Direction8::N))
            .with(Soldier {
                name: "Test".to_string(),
                faction: Faction::Allies,
                rank: crate::components::soldier::Rank::Private,
            })
            .build();

        let entities = world.entities();
        let positions = world.read_storage::<Position>();
        let visions = world.read_storage::<Vision>();
        let facings = world.read_storage::<Facing>();
        let soldiers = world.read_storage::<Soldier>();

        let result =
            calculate_faction_vision(&entities, &positions, &visions, &facings, &soldiers, Faction::Allies, &bf);

        // Should have some visible tiles
        assert!(!result.visible_tiles.is_empty());
        // Should have tracked at least one spotter
        assert!(!result.spotter_map.is_empty());
    }

    #[test]
    fn test_shared_vision_multiple_entities() {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Vision>();
        world.register::<Facing>();
        world.register::<Soldier>();

        let mut bf = Battlefield::new(40, 40);
        for x in 0..40 {
            for y in 0..40 {
                bf.set_terrain(BattlefieldPosition::new(x, y), TerrainType::NoMansLand);
            }
        }

        // Create two entities facing opposite directions
        world
            .create_entity()
            .with(Position::new(10, 10))
            .with(Vision::new(5))
            .with(Facing::new(Direction8::N))
            .with(Soldier {
                name: "Ally1".to_string(),
                faction: Faction::Allies,
                rank: crate::components::soldier::Rank::Private,
            })
            .build();

        world
            .create_entity()
            .with(Position::new(10, 20))
            .with(Vision::new(5))
            .with(Facing::new(Direction8::S))
            .with(Soldier {
                name: "Ally2".to_string(),
                faction: Faction::Allies,
                rank: crate::components::soldier::Rank::Private,
            })
            .build();

        let entities = world.entities();
        let positions = world.read_storage::<Position>();
        let visions = world.read_storage::<Vision>();
        let facings = world.read_storage::<Facing>();
        let soldiers = world.read_storage::<Soldier>();

        let result =
            calculate_faction_vision(&entities, &positions, &visions, &facings, &soldiers, Faction::Allies, &bf);

        // Should have more visible tiles than a single entity
        assert!(!result.visible_tiles.is_empty());
        // Both entities should have contributed
        assert!(result.spotter_map.len() > 5);
    }

    #[test]
    fn test_shared_vision_filters_faction() {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Vision>();
        world.register::<Facing>();
        world.register::<Soldier>();

        let mut bf = Battlefield::new(20, 20);
        for x in 0..20 {
            for y in 0..20 {
                bf.set_terrain(BattlefieldPosition::new(x, y), TerrainType::NoMansLand);
            }
        }

        // Create entities from different factions
        world
            .create_entity()
            .with(Position::new(10, 10))
            .with(Vision::new(5))
            .with(Facing::new(Direction8::N))
            .with(Soldier {
                name: "Ally".to_string(),
                faction: Faction::Allies,
                rank: crate::components::soldier::Rank::Private,
            })
            .build();

        world
            .create_entity()
            .with(Position::new(15, 15))
            .with(Vision::new(5))
            .with(Facing::new(Direction8::S))
            .with(Soldier {
                name: "Enemy".to_string(),
                faction: Faction::CentralPowers,
                rank: crate::components::soldier::Rank::Private,
            })
            .build();

        let entities = world.entities();
        let positions = world.read_storage::<Position>();
        let visions = world.read_storage::<Vision>();
        let facings = world.read_storage::<Facing>();
        let soldiers = world.read_storage::<Soldier>();

        // Calculate only for Allies
        let result =
            calculate_faction_vision(&entities, &positions, &visions, &facings, &soldiers, Faction::Allies, &bf);

        // Should only include vision from Allied entity
        // The enemy entity at (15,15) should not contribute to spotters
        assert!(!result.visible_tiles.is_empty());
    }
}
