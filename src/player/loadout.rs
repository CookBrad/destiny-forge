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

/// Weapons and armor owned but not equipped. Hub-only swap; never mid-dungeon.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GearStash {
    pub weapons: Vec<WeaponKind>,
    pub armor: Vec<ArmorKind>,
}

#[derive(Resource, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Loadout {
    pub weapon: WeaponKind,
    pub armor: ArmorSlots,
    /// Additive so old profiles deserialize without this field.
    #[serde(default)]
    pub stash: GearStash,
}

impl Default for Loadout {
    fn default() -> Self {
        Self {
            weapon: WeaponKind::RustySword,
            armor: ArmorSlots::default(),
            stash: GearStash::default(),
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

    fn piece_mut(&mut self, slot: ArmorSlot) -> &mut Option<ArmorKind> {
        match slot {
            ArmorSlot::Head => &mut self.head,
            ArmorSlot::Chest => &mut self.chest,
            ArmorSlot::Arms => &mut self.arms,
            ArmorSlot::Legs => &mut self.legs,
        }
    }
}

impl Loadout {
    pub fn equipped_weapon(&self) -> EquippedWeapon {
        EquippedWeapon(self.weapon)
    }

    pub fn weapon_label(&self) -> &'static str {
        weapon_kind_label(self.weapon)
    }

    /// Forge an alternate weapon: previous goes to stash unless this is an upgrade consume.
    pub fn equip_alternate_weapon(&mut self, weapon: WeaponKind) {
        if self.weapon == weapon {
            return;
        }
        self.store_weapon(self.weapon);
        self.take_weapon_from_stash(weapon);
        self.weapon = weapon;
    }

    /// Forge upgrade that consumes the equipped weapon (do not stash it).
    pub fn upgrade_weapon(&mut self, weapon: WeaponKind) {
        self.take_weapon_from_stash(weapon);
        self.weapon = weapon;
    }

    /// Equip a stashed weapon at the hub. Returns false if it is not in the stash.
    pub fn swap_to_stashed_weapon(&mut self, weapon: WeaponKind) -> bool {
        if self.weapon == weapon || !self.stash.weapons.contains(&weapon) {
            return false;
        }
        self.store_weapon(self.weapon);
        self.take_weapon_from_stash(weapon);
        self.weapon = weapon;
        true
    }

    pub fn equip_forged_armor(&mut self, kind: ArmorKind) {
        let slot = kind.slot();
        if let Some(previous) = *self.armor.piece_mut(slot) {
            if previous != kind {
                self.store_armor(previous);
            }
        }
        self.take_armor_from_stash(kind);
        self.armor.set(kind);
    }

    pub fn swap_to_stashed_armor(&mut self, kind: ArmorKind) -> bool {
        if !self.stash.armor.contains(&kind) {
            return false;
        }
        self.equip_forged_armor(kind);
        true
    }

    fn store_weapon(&mut self, weapon: WeaponKind) {
        if !self.stash.weapons.contains(&weapon) {
            self.stash.weapons.push(weapon);
        }
    }

    fn take_weapon_from_stash(&mut self, weapon: WeaponKind) {
        self.stash.weapons.retain(|stored| *stored != weapon);
    }

    fn store_armor(&mut self, kind: ArmorKind) {
        if !self.stash.armor.contains(&kind) {
            self.stash.armor.push(kind);
        }
    }

    fn take_armor_from_stash(&mut self, kind: ArmorKind) {
        self.stash.armor.retain(|stored| *stored != kind);
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

pub fn weapon_kind_label(kind: WeaponKind) -> &'static str {
    match kind {
        WeaponKind::RustySword => "Rusty Sword",
        WeaponKind::RustySpear => "Rusty Spear",
        WeaponKind::IronSword => "Iron Sword",
        WeaponKind::SlimeBlade => "Slime Blade",
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
        assert!((loadout.carve_speed_multiplier() - 1.1).abs() < f32::EPSILON);
    }

    #[test]
    fn alternate_weapon_stashes_previous() {
        let mut loadout = Loadout::default();
        loadout.equip_alternate_weapon(WeaponKind::RustySpear);
        assert_eq!(loadout.weapon, WeaponKind::RustySpear);
        assert_eq!(loadout.stash.weapons, vec![WeaponKind::RustySword]);
    }

    #[test]
    fn upgrade_does_not_stash_consumed_weapon() {
        let mut loadout = Loadout::default();
        loadout.weapon = WeaponKind::IronSword;
        loadout.upgrade_weapon(WeaponKind::SlimeBlade);
        assert_eq!(loadout.weapon, WeaponKind::SlimeBlade);
        assert!(!loadout.stash.weapons.contains(&WeaponKind::IronSword));
    }

    #[test]
    fn hub_swap_exchanges_equipped_and_stash() {
        let mut loadout = Loadout::default();
        loadout.equip_alternate_weapon(WeaponKind::RustySpear);
        assert!(loadout.swap_to_stashed_weapon(WeaponKind::RustySword));
        assert_eq!(loadout.weapon, WeaponKind::RustySword);
        assert_eq!(loadout.stash.weapons, vec![WeaponKind::RustySpear]);
    }

    #[test]
    fn swap_rejects_weapon_not_in_stash() {
        let mut loadout = Loadout::default();
        assert!(!loadout.swap_to_stashed_weapon(WeaponKind::IronSword));
        assert_eq!(loadout.weapon, WeaponKind::RustySword);
    }

    #[test]
    fn forged_armor_stashes_previous_piece_in_slot() {
        let mut loadout = Loadout::default();
        loadout.equip_forged_armor(ArmorKind::SlimeHelm);
        assert_eq!(loadout.armor.head, Some(ArmorKind::SlimeHelm));
        assert!(loadout.stash.armor.is_empty());
    }
}
