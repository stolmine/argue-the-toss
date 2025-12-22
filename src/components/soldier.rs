// Soldier component for individual units

use specs::{Component, VecStorage};

/// Represents a soldier unit on the battlefield
#[derive(Debug, Clone)]
pub struct Soldier {
    pub name: String,
    pub faction: Faction,
    pub rank: Rank,
}

impl Component for Soldier {
    type Storage = VecStorage<Self>;
}

/// Faction/side the soldier belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Faction {
    Allies,
    CentralPowers,
}

impl Faction {
    /// Returns the display character for this faction
    pub fn to_char(&self) -> char {
        match self {
            Faction::Allies => '@',
            Faction::CentralPowers => 'Ӝ',
        }
    }
}

/// Military rank of the soldier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    Private,
    Corporal,
    Sergeant,
    Lieutenant,
    Captain,
}

pub struct RankBaseStats {
    pub base_hp: i32,
    pub vision_range: i32,
    pub accuracy_base: f32,
    pub movement_speed_base: f32,
    pub carrying_capacity_base: i32,
}

impl Rank {
    pub fn as_str(&self) -> &'static str {
        match self {
            Rank::Private => "Pvt",
            Rank::Corporal => "Cpl",
            Rank::Sergeant => "Sgt",
            Rank::Lieutenant => "Lt",
            Rank::Captain => "Cpt",
        }
    }

    pub fn base_stats(&self) -> RankBaseStats {
        match self {
            Rank::Private => RankBaseStats {
                base_hp: 100,
                vision_range: 10,
                accuracy_base: 0.0,
                movement_speed_base: 1.0,
                carrying_capacity_base: 20,
            },
            Rank::Corporal => RankBaseStats {
                base_hp: 110,
                vision_range: 11,
                accuracy_base: 0.05,
                movement_speed_base: 1.05,
                carrying_capacity_base: 22,
            },
            Rank::Sergeant => RankBaseStats {
                base_hp: 120,
                vision_range: 12,
                accuracy_base: 0.10,
                movement_speed_base: 1.10,
                carrying_capacity_base: 25,
            },
            Rank::Lieutenant => RankBaseStats {
                base_hp: 130,
                vision_range: 13,
                accuracy_base: 0.15,
                movement_speed_base: 1.15,
                carrying_capacity_base: 28,
            },
            Rank::Captain => RankBaseStats {
                base_hp: 140,
                vision_range: 15,
                accuracy_base: 0.20,
                movement_speed_base: 1.20,
                carrying_capacity_base: 30,
            },
        }
    }

    pub fn to_icon(&self) -> char {
        match self {
            Rank::Captain => '★',
            Rank::Lieutenant => '☆',
            Rank::Sergeant => '●',
            Rank::Corporal => '○',
            Rank::Private => '■',
        }
    }

    pub fn distribution_weight(&self) -> u32 {
        match self {
            Rank::Captain => 2,
            Rank::Lieutenant => 3,
            Rank::Sergeant => 10,
            Rank::Corporal => 15,
            Rank::Private => 70,
        }
    }

    pub fn all() -> [Rank; 5] {
        [
            Rank::Captain,
            Rank::Lieutenant,
            Rank::Sergeant,
            Rank::Corporal,
            Rank::Private,
        ]
    }
}
