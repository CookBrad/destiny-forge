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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmorSlot {
    Head,
    Chest,
    Arms,
    Legs,
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

impl ArmorKind {
    pub fn slot(self) -> ArmorSlot {
        match self {
            Self::SlimeHelm => ArmorSlot::Head,
            Self::SlimeMail => ArmorSlot::Chest,
            Self::SlimeGauntlets => ArmorSlot::Arms,
            Self::SlimeGreaves => ArmorSlot::Legs,
        }
    }

    pub fn defense(self) -> f32 {
        match self {
            Self::SlimeHelm => 2.0,
            Self::SlimeMail => 4.0,
            Self::SlimeGauntlets => 1.0,
            Self::SlimeGreaves => 2.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::SlimeHelm => "Slime Helm",
            Self::SlimeMail => "Slime Mail",
            Self::SlimeGauntlets => "Slime Gauntlets",
            Self::SlimeGreaves => "Slime Greaves",
        }
    }
}

impl ArmorSlots {
    pub fn set(&mut self, kind: ArmorKind) {
        match kind.slot() {
            ArmorSlot::Head => self.head = Some(kind),
            ArmorSlot::Chest => self.chest = Some(kind),
            ArmorSlot::Arms => self.arms = Some(kind),
            ArmorSlot::Legs => self.legs = Some(kind),
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
            WeaponKind::SlimeBlade => "Slime Blade",
        }
    }

    pub fn total_defense(&self) -> f32 {
        let mut total = 0.0;
        if let Some(kind) = self.armor.head {
            total += kind.defense();
        }
        if let Some(kind) = self.armor.chest {
            total += kind.defense();
        }
        if let Some(kind) = self.armor.arms {
            total += kind.defense();
        }
        if let Some(kind) = self.armor.legs {
            total += kind.defense();
        }
        total
    }

    pub fn slime_set_pieces(&self) -> u32 {
        [self.armor.head, self.armor.chest, self.armor.arms, self.armor.legs]
            .into_iter()
            .filter(|piece| piece.is_some())
            .count() as u32
    }

    pub fn carve_speed_multiplier(&self) -> f32 {
        if self.slime_set_pieces() >= 2 {
            1.1
        } else {
            1.0
        }
    }

    /// 2pc combat skill: special cooldowns resolve slightly faster.
    pub fn special_cooldown_multiplier(&self) -> f32 {
        if self.slime_set_pieces() >= 2 {
            0.9
        } else {
            1.0
        }
    }

    pub fn knockback_resist(&self) -> f32 {
        if self.slime_set_pieces() >= 4 {
            0.35
        } else {
            0.0
        }
    }

    /// 4pc combat skill: +10% attack power on weapons and specials.
    pub fn attack_power_multiplier(&self) -> f32 {
        if self.slime_set_pieces() >= 4 {
            1.1
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_slime() -> Loadout {
        let mut loadout = Loadout::default();
        loadout.armor.head = Some(ArmorKind::SlimeHelm);
        loadout.armor.chest = Some(ArmorKind::SlimeMail);
        loadout.armor.arms = Some(ArmorKind::SlimeGauntlets);
        loadout.armor.legs = Some(ArmorKind::SlimeGreaves);
        loadout
    }

    #[test]
    fn two_piece_gives_carve_and_special_cd() {
        let mut loadout = Loadout::default();
        loadout.armor.head = Some(ArmorKind::SlimeHelm);
        loadout.armor.chest = Some(ArmorKind::SlimeMail);
        assert_eq!(loadout.slime_set_pieces(), 2);
        assert!((loadout.carve_speed_multiplier() - 1.1).abs() < f32::EPSILON);
        assert!((loadout.special_cooldown_multiplier() - 0.9).abs() < f32::EPSILON);
        assert!((loadout.attack_power_multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn four_piece_includes_attack_and_knockback() {
        let loadout = full_slime();
        assert_eq!(loadout.slime_set_pieces(), 4);
        assert!((loadout.knockback_resist() - 0.35).abs() < f32::EPSILON);
        assert!((loadout.attack_power_multiplier() - 1.1).abs() < f32::EPSILON);
        // 4pc keeps 2pc carve bonus
        assert!((loadout.carve_speed_multiplier() - 1.1).abs() < f32::EPSILON);
    }
}