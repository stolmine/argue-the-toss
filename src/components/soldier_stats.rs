use specs::{Component, VecStorage};

#[derive(Debug, Clone)]
pub struct SoldierStats {
    pub accuracy_modifier: f32,
    pub movement_speed_modifier: f32,
    pub max_hp_modifier: i32,
    pub carrying_capacity: i32,
}

impl Component for SoldierStats {
    type Storage = VecStorage<Self>;
}

impl SoldierStats {
    pub fn new(
        accuracy_modifier: f32,
        movement_speed_modifier: f32,
        max_hp_modifier: i32,
        carrying_capacity: i32,
    ) -> Self {
        Self {
            accuracy_modifier,
            movement_speed_modifier,
            max_hp_modifier,
            carrying_capacity,
        }
    }

    pub fn default_for_rank(rank: &super::soldier::Rank) -> Self {
        let base = rank.base_stats();
        Self {
            accuracy_modifier: base.accuracy_base,
            movement_speed_modifier: base.movement_speed_base,
            max_hp_modifier: 0,
            carrying_capacity: base.carrying_capacity_base,
        }
    }
}
