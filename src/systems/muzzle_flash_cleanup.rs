// System to clear muzzle flash effects after one frame

use crate::components::muzzle_flash::MuzzleFlash;
use specs::{Entities, Join, System, WriteStorage};

pub struct MuzzleFlashCleanupSystem;

impl<'a> System<'a> for MuzzleFlashCleanupSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, MuzzleFlash>,
    );

    fn run(&mut self, (entities, mut flashes): Self::SystemData) {
        // Collect entities with muzzle flashes
        let to_remove: Vec<_> = (&entities, &flashes)
            .join()
            .map(|(entity, _flash)| entity)
            .collect();

        // Remove all muzzle flashes
        for entity in to_remove {
            flashes.remove(entity);
        }
    }
}
