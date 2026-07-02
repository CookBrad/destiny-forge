use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct EquippedWeapon(pub WeaponKind);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeaponKind {
    #[default]
    RustySword,
    RustySpear,
}

#[derive(Clone, Copy, Debug)]
pub struct WeaponStats {
    pub attack_power: f32,
    /// Pixels forward from the player center (spear); sword uses sprite geometry instead.
    pub reach: f32,
    pub swing_secs: f32,
    pub hit_start: f32,
    pub hit_end: f32,
}

impl WeaponKind {
    pub fn stats(self) -> WeaponStats {
        match self {
            Self::RustySword => WeaponStats {
                attack_power: 10.0,
                reach: 30.0,
                swing_secs: 0.28,
                hit_start: 0.08,
                hit_end: 0.18,
            },
            Self::RustySpear => WeaponStats {
                attack_power: 12.0,
                reach: 38.0,
                swing_secs: 0.36,
                hit_start: 0.12,
                hit_end: 0.24,
            },
        }
    }
}