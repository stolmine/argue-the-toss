// Weapon components for combat system
// Data-driven design for easy extensibility

use specs::{Component, VecStorage};

/// Type of weapon - determines weapon stats via factory functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponType {
    /// WWI standard rifle (Lee-Enfield, Gewehr 98, etc.)
    Rifle,
    /// Submachine gun (MP18, etc.) - closer range, higher ROF
    SubmachineGun,
    /// Light machine gun (Lewis, MG08, etc.) - sustained fire
    MachineGun,
    /// Pistol/revolver - close range backup
    Pistol,
}

impl WeaponType {
    /// Get default stats for this weapon type
    /// This is the factory pattern - adding new weapons is just data
    pub fn default_stats(&self) -> WeaponStats {
        match self {
            WeaponType::Rifle => WeaponStats {
                name: "Rifle".to_string(),
                max_range: 30,        // tiles
                effective_range: 15,  // optimal accuracy within this range
                base_accuracy: 0.7,   // 70% hit chance at effective range
                damage: 25,           // enough to kill in 3-4 hits
                fire_time: 3.0,       // seconds per shot
                reload_time: 5.0,     // seconds to reload
            },
            WeaponType::SubmachineGun => WeaponStats {
                name: "SMG".to_string(),
                max_range: 15,
                effective_range: 8,
                base_accuracy: 0.6,   // less accurate but faster
                damage: 18,
                fire_time: 2.0,       // faster fire rate
                reload_time: 4.0,
            },
            WeaponType::MachineGun => WeaponStats {
                name: "Machine Gun".to_string(),
                max_range: 40,
                effective_range: 20,
                base_accuracy: 0.8,   // very accurate when set up
                damage: 30,
                fire_time: 2.5,
                reload_time: 8.0,     // long reload
            },
            WeaponType::Pistol => WeaponStats {
                name: "Pistol".to_string(),
                max_range: 10,
                effective_range: 5,
                base_accuracy: 0.5,   // not very accurate
                damage: 15,
                fire_time: 2.0,
                reload_time: 3.0,
            },
        }
    }
}

/// Weapon statistics (data-driven)
#[derive(Debug, Clone)]
pub struct WeaponStats {
    pub name: String,
    pub max_range: i32,       // Maximum range in tiles
    pub effective_range: i32, // Range where base_accuracy applies
    pub base_accuracy: f32,   // Hit chance at effective range (0.0-1.0)
    pub damage: i32,          // Base damage per hit
    pub fire_time: f32,       // Time cost to fire (seconds)
    pub reload_time: f32,     // Time cost to reload (seconds)
}

/// Ammunition state for a weapon
#[derive(Debug, Clone)]
pub struct AmmoState {
    pub current: i32,
    pub max_capacity: i32,
}

impl AmmoState {
    pub fn new(capacity: i32) -> Self {
        Self {
            current: capacity,
            max_capacity: capacity,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.current <= 0
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.max_capacity
    }

    pub fn consume(&mut self, amount: i32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    pub fn reload(&mut self) {
        self.current = self.max_capacity;
    }

    pub fn percentage(&self) -> f32 {
        if self.max_capacity == 0 {
            0.0
        } else {
            (self.current as f32 / self.max_capacity as f32) * 100.0
        }
    }
}

/// Component: Weapon equipped by an entity
#[derive(Debug, Clone)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub stats: WeaponStats,
    pub ammo: AmmoState,
}

impl Component for Weapon {
    type Storage = VecStorage<Self>;
}

impl Weapon {
    /// Create a new weapon of the given type
    /// This is the main factory function - makes adding weapons trivial
    pub fn new(weapon_type: WeaponType, ammo_capacity: i32) -> Self {
        let stats = weapon_type.default_stats();
        Self {
            weapon_type,
            stats,
            ammo: AmmoState::new(ammo_capacity),
        }
    }

    /// Convenience factory: Standard rifle with 10 rounds
    pub fn rifle() -> Self {
        Self::new(WeaponType::Rifle, 10)
    }

    /// Convenience factory: SMG with 32 rounds
    pub fn smg() -> Self {
        Self::new(WeaponType::SubmachineGun, 32)
    }

    /// Convenience factory: Machine gun with 100 rounds
    pub fn machine_gun() -> Self {
        Self::new(WeaponType::MachineGun, 100)
    }

    /// Convenience factory: Pistol with 8 rounds
    pub fn pistol() -> Self {
        Self::new(WeaponType::Pistol, 8)
    }

    /// Check if weapon can fire
    pub fn can_fire(&self) -> bool {
        !self.ammo.is_empty()
    }

    /// Consume ammo for firing
    pub fn fire(&mut self) -> bool {
        self.ammo.consume(1)
    }

    /// Reload weapon
    pub fn reload(&mut self) {
        self.ammo.reload();
    }
}
