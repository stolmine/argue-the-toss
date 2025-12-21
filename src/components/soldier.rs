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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    Private,
    Corporal,
    Sergeant,
    Lieutenant,
    Captain,
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
}
