use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::combat::{EquippedWeapon, WeaponKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmorKind {
    SlimeHelm,
    SlimeMail,
    SlimeGauntlets,
    SlimeGreaves,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmorSlots {
    pub head: Option<ArmorKind>,
    pub chest: Option<ArmorKind>,
    pub arms: Option<ArmorKind>,
    pub legs: Option<ArmorKind>,
}

#[derive(Resource, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Loadout {
    pub weapon: WeaponKind,
    pub armor: ArmorSlots,
}

impl Default for Loadout {
    fn default() -> Self {
        Self {
            weapon: WeaponKind::RustySword,
            armor: ArmorSlots::default(),
        }
    }
}

impl Loadout {
    pub fn equipped_weapon(&self) -> EquippedWeapon {
        EquippedWeapon(self.weapon)
    }

    pub fn weapon_label(&self) -> &'static str {
        match self.weapon {
            WeaponKind::RustySword => "Rusty Sword",
            WeaponKind::RustySpear => "Rusty Spear",
            WeaponKind::IronSword => "Iron Sword",
        }
    }
}