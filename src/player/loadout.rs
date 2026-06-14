use bevy::prelude::*;

use crate::items::{ArmorId, WeaponId};
use crate::progression::SlimeSet;

#[derive(Resource, Clone, Debug)]
pub struct PlayerLoadout {
    pub weapon: WeaponId,
    pub armor: [Option<ArmorId>; 4],
}

impl Default for PlayerLoadout {
    fn default() -> Self {
        Self {
            weapon: WeaponId::RustySword,
            armor: [None; 4],
        }
    }
}

impl PlayerLoadout {
    pub fn equip_weapon(&mut self, weapon: WeaponId) {
        self.weapon = weapon;
    }

    pub fn equip_armor(&mut self, armor: ArmorId) {
        let index = SlimeSet::slot_index(armor.slot());
        self.armor[index] = Some(armor);
    }

    pub fn total_defense(&self) -> f32 {
        SlimeSet::total_defense_bonus(&self.armor)
    }

    pub fn weapon_damage(&self) -> f32 {
        self.weapon.damage()
    }

    pub fn weapon_reach(&self) -> f32 {
        self.weapon.reach()
    }

    pub fn carve_speed_multiplier(&self) -> f32 {
        SlimeSet::active_bonuses(&self.armor)
            .iter()
            .map(|bonus| bonus.carve_speed_multiplier)
            .fold(1.0, |total, multiplier| total.max(multiplier))
    }

    pub fn knockback_resistance(&self) -> f32 {
        SlimeSet::active_bonuses(&self.armor)
            .iter()
            .map(|bonus| bonus.knockback_resistance)
            .fold(0.0, f32::max)
    }

    pub fn owns_weapon(&self, weapon: WeaponId) -> bool {
        self.weapon == weapon
    }

    pub fn owns_armor(&self, armor: ArmorId) -> bool {
        self.armor.iter().any(|piece| *piece == Some(armor))
    }
}