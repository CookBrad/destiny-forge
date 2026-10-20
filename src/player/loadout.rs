use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::combat::{EquippedWeapon, WeaponKind};
use crate::items::{FoodBuff, Inventory, MaterialId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmorSet {
    Helm,
    Mail,
    Gauntlets,
    Greaves,
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
    pub head: Option<ArmorSet>,
    pub chest: Option<ArmorSet>,
    pub arms: Option<ArmorSet>,
    pub legs: Option<ArmorSet>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GearStash {
    pub weapons: Vec<WeaponKind>,
    pub armor: Vec<ArmorSet>,
}

#[derive(Resource, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Loadout {
    pub weapon: WeaponKind,
    pub armor: ArmorSlots,
    /// Additive so old profiles deserialize without this field.
    #[serde(default)]
    pub stash: GearStash,
    #[serde(default)]
    pub food_buff: Option<MaterialId>,
}

impl Default for Loadout {
    fn default() -> Self {
        Self {
            weapon: WeaponKind::RustySword,
            armor: ArmorSlots::default(),
            stash: GearStash::default(),
            food_buff: None,
        }
    }
}

impl ArmorSet {
    pub fn slot(self) -> ArmorSlot {
        match self {
            Self::Helm => ArmorSlot::Head,
            Self::Mail => ArmorSlot::Chest,
            Self::Gauntlets => ArmorSlot::Arms,
            Self::Greaves => ArmorSlot::Legs,
        }
    }

    pub fn defense(self) -> f32 {
        match self {
            Self::Helm => 2.0,
            Self::Mail => 4.0,
            Self::Gauntlets => 1.0,
            Self::Greaves => 2.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Helm => "Slime Helm",
            Self::Mail => "Slime Mail",
            Self::Gauntlets => "Slime Gauntlets",
            Self::Greaves => "Slime Greaves",
        }
    }
}

impl ArmorSlots {
    pub fn set(&mut self, kind: ArmorSet) {
        match kind.slot() {
            ArmorSlot::Head => self.head = Some(kind),
            ArmorSlot::Chest => self.chest = Some(kind),
            ArmorSlot::Arms => self.arms = Some(kind),
            ArmorSlot::Legs => self.legs = Some(kind),
        }
    }

    fn piece_mut(&mut self, slot: ArmorSlot) -> &mut Option<ArmorSet> {
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

    pub fn equip_forged_armor(&mut self, kind: ArmorSet) {
        let slot = kind.slot();
        if let Some(previous) = *self.armor.piece_mut(slot) {
            if previous != kind {
                self.store_armor(previous);
            }
        }
        self.take_armor_from_stash(kind);
        self.armor.set(kind);
    }

    pub fn swap_to_stashed_armor(&mut self, kind: ArmorSet) -> bool {
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

    fn store_armor(&mut self, kind: ArmorSet) {
        if !self.stash.armor.contains(&kind) {
            self.stash.armor.push(kind);
        }
    }

    fn take_armor_from_stash(&mut self, kind: ArmorSet) {
        self.stash.armor.retain(|stored| *stored != kind);
    }

    pub fn eat_food_replacing_active_buff(
        &mut self,
        inventory: &mut Inventory,
        food: MaterialId,
    ) -> bool {
        if food.food_buff().is_none() || !inventory.try_remove(food, 1) {
            return false;
        }
        self.food_buff = Some(food);
        true
    }

    pub fn clear_food_buff(&mut self) {
        if self.food_buff.is_some() {
            self.food_buff = None;
        }
    }

    pub fn active_food_buff(&self) -> FoodBuff {
        self.food_buff
            .and_then(MaterialId::food_buff)
            .unwrap_or(FoodBuff::NONE)
    }

    pub fn total_defense(&self) -> f32 {
        let mut total = self.active_food_buff().defense_bonus;
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
        [
            self.armor.head,
            self.armor.chest,
            self.armor.arms,
            self.armor.legs,
        ]
        .into_iter()
        .filter(|piece| piece.is_some())
        .count() as u32
    }

    pub fn carve_speed_multiplier(&self) -> f32 {
        let slime_set_carve_multiplier = if self.slime_set_pieces() >= 2 {
            1.1
        } else {
            1.0
        };
        slime_set_carve_multiplier * self.active_food_buff().carve_speed_multiplier
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
        let slime_set_attack_multiplier = if self.slime_set_pieces() >= 4 {
            1.1
        } else {
            1.0
        };
        slime_set_attack_multiplier * self.active_food_buff().attack_multiplier
    }
}

pub fn expire_food_buff_after_hunt(mut loadout: ResMut<Loadout>) {
    loadout.clear_food_buff();
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
    use bevy::ecs::system::RunSystemOnce;

    fn full_slime() -> Loadout {
        let mut loadout = Loadout::default();
        loadout.armor.head = Some(ArmorSet::Helm);
        loadout.armor.chest = Some(ArmorSet::Mail);
        loadout.armor.arms = Some(ArmorSet::Gauntlets);
        loadout.armor.legs = Some(ArmorSet::Greaves);
        loadout
    }

    #[test]
    fn two_piece_gives_carve_and_special_cd() {
        let mut loadout = Loadout::default();
        loadout.armor.head = Some(ArmorSet::Helm);
        loadout.armor.chest = Some(ArmorSet::Mail);
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
        let mut loadout = Loadout {
            weapon: WeaponKind::IronSword,
            ..Loadout::default()
        };
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
    fn eating_consumes_one_food_and_sets_buff() {
        let mut loadout = Loadout::default();
        let mut inventory = Inventory::default();
        inventory.try_add(MaterialId::RoastTurnip, 2);

        assert!(loadout.eat_food_replacing_active_buff(&mut inventory, MaterialId::RoastTurnip));
        assert_eq!(inventory.count(MaterialId::RoastTurnip), 1);
        assert_eq!(loadout.food_buff, Some(MaterialId::RoastTurnip));
    }

    #[test]
    fn eating_rejects_non_food_and_missing_food() {
        let mut loadout = Loadout::default();
        let mut inventory = Inventory::default();
        inventory.try_add(MaterialId::Turnip, 1);

        assert!(!loadout.eat_food_replacing_active_buff(&mut inventory, MaterialId::Turnip));
        assert!(!loadout.eat_food_replacing_active_buff(&mut inventory, MaterialId::PotatoStew));
        assert_eq!(inventory.count(MaterialId::Turnip), 1);
        assert_eq!(loadout.food_buff, None);
    }

    #[test]
    fn eating_again_replaces_buff_instead_of_stacking() {
        let mut loadout = Loadout::default();
        let mut inventory = Inventory::default();
        inventory.try_add(MaterialId::RoastTurnip, 1);
        inventory.try_add(MaterialId::PotatoStew, 1);

        assert!(loadout.eat_food_replacing_active_buff(&mut inventory, MaterialId::RoastTurnip));
        assert!(loadout.eat_food_replacing_active_buff(&mut inventory, MaterialId::PotatoStew));
        assert_eq!(loadout.food_buff, Some(MaterialId::PotatoStew));
        assert!((loadout.attack_power_multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn food_buffs_fold_into_combat_multipliers() {
        let mut loadout = full_slime();
        loadout.food_buff = Some(MaterialId::RoastTurnip);
        assert!((loadout.attack_power_multiplier() - 1.1 * 1.08).abs() < 1e-5);

        loadout.food_buff = Some(MaterialId::PotatoStew);
        assert!((loadout.total_defense() - (9.0 + 2.0)).abs() < f32::EPSILON);
        assert!((loadout.carve_speed_multiplier() - 1.1 * 1.1).abs() < 1e-5);
        assert!((loadout.attack_power_multiplier() - 1.1).abs() < f32::EPSILON);
    }

    #[test]
    fn food_buff_ends_when_leaving_the_dungeon() {
        let mut app = App::new();
        let loadout = Loadout {
            food_buff: Some(MaterialId::PotatoStew),
            ..Loadout::default()
        };
        app.insert_resource(loadout);

        app.world_mut()
            .run_system_once(expire_food_buff_after_hunt)
            .unwrap();

        assert_eq!(app.world().resource::<Loadout>().food_buff, None);
    }

    #[test]
    fn old_loadout_without_food_buff_field_loads() {
        let old = "(weapon: RustySword, armor: (head: None, chest: None, arms: None, legs: None))";
        let loadout: Loadout = ron::from_str(old).unwrap();
        assert_eq!(loadout.food_buff, None);
    }

    #[test]
    fn forged_armor_stashes_previous_piece_in_slot() {
        let mut loadout = Loadout::default();
        loadout.equip_forged_armor(ArmorSet::Helm);
        assert_eq!(loadout.armor.head, Some(ArmorSet::Helm));
        assert!(loadout.stash.armor.is_empty());
    }

    #[test]
    fn armor_slots_place_each_piece_in_its_own_slot() {
        let mut slots = ArmorSlots::default();
        slots.set(ArmorSet::Helm);
        slots.set(ArmorSet::Mail);
        slots.set(ArmorSet::Gauntlets);
        slots.set(ArmorSet::Greaves);
        assert_eq!(slots.head, Some(ArmorSet::Helm));
        assert_eq!(slots.chest, Some(ArmorSet::Mail));
        assert_eq!(slots.arms, Some(ArmorSet::Gauntlets));
        assert_eq!(slots.legs, Some(ArmorSet::Greaves));
    }

    #[test]
    fn armor_slots_replace_only_the_slot_that_piece_uses() {
        let mut slots = ArmorSlots {
            head: Some(ArmorSet::Helm),
            chest: Some(ArmorSet::Mail),
            arms: Some(ArmorSet::Gauntlets),
            legs: Some(ArmorSet::Greaves),
        };
        slots.set(ArmorSet::Mail);
        assert_eq!(slots.head, Some(ArmorSet::Helm));
        assert_eq!(slots.chest, Some(ArmorSet::Mail));
        assert_eq!(slots.arms, Some(ArmorSet::Gauntlets));
        assert_eq!(slots.legs, Some(ArmorSet::Greaves));
    }

    #[test]
    fn armor_slots_leave_the_other_slots_empty() {
        let mut slots = ArmorSlots::default();
        slots.set(ArmorSet::Greaves);
        assert_eq!(slots.legs, Some(ArmorSet::Greaves));
        assert_eq!(slots.head, None);
        assert_eq!(slots.chest, None);
        assert_eq!(slots.arms, None);
    }

    #[test]
    fn forging_a_second_piece_stashes_the_piece_already_in_that_slot() {
        let mut loadout = Loadout::default();
        loadout.armor.head = Some(ArmorSet::Helm);
        loadout.equip_forged_armor(ArmorSet::Helm);
        assert_eq!(loadout.armor.head, Some(ArmorSet::Helm));
        assert!(loadout.stash.armor.is_empty());
        loadout.stash.armor.push(ArmorSet::Mail);
        loadout.equip_forged_armor(ArmorSet::Mail);
        assert_eq!(loadout.armor.chest, Some(ArmorSet::Mail));
        assert!(loadout.stash.armor.is_empty());
    }

    #[test]
    fn swapping_to_stashed_armor_equips_it_and_clears_it_from_the_stash() {
        let mut loadout = Loadout::default();
        loadout.stash.armor.push(ArmorSet::Gauntlets);
        assert!(loadout.swap_to_stashed_armor(ArmorSet::Gauntlets));
        assert_eq!(loadout.armor.arms, Some(ArmorSet::Gauntlets));
        assert!(loadout.stash.armor.is_empty());
    }

    #[test]
    fn swapping_armor_that_is_not_in_the_stash_leaves_the_loadout_unchanged() {
        let mut loadout = Loadout::default();
        loadout.armor.legs = Some(ArmorSet::Greaves);
        assert!(!loadout.swap_to_stashed_armor(ArmorSet::Greaves));
        assert_eq!(loadout.armor.legs, Some(ArmorSet::Greaves));
        assert!(loadout.stash.armor.is_empty());
    }

    #[test]
    fn each_slot_borrow_points_at_that_slot() {
        let mut slots = ArmorSlots::default();
        *slots.piece_mut(ArmorSlot::Head) = Some(ArmorSet::Helm);
        *slots.piece_mut(ArmorSlot::Chest) = Some(ArmorSet::Mail);
        *slots.piece_mut(ArmorSlot::Arms) = Some(ArmorSet::Gauntlets);
        *slots.piece_mut(ArmorSlot::Legs) = Some(ArmorSet::Greaves);
        assert_eq!(*slots.piece_mut(ArmorSlot::Head), Some(ArmorSet::Helm));
        assert_eq!(*slots.piece_mut(ArmorSlot::Chest), Some(ArmorSet::Mail));
        assert_eq!(*slots.piece_mut(ArmorSlot::Arms), Some(ArmorSet::Gauntlets));
        assert_eq!(*slots.piece_mut(ArmorSlot::Legs), Some(ArmorSet::Greaves));
    }

    #[test]
    fn clearing_one_slot_borrow_leaves_the_other_slots() {
        let mut slots = ArmorSlots {
            head: Some(ArmorSet::Helm),
            chest: Some(ArmorSet::Mail),
            arms: Some(ArmorSet::Gauntlets),
            legs: Some(ArmorSet::Greaves),
        };
        *slots.piece_mut(ArmorSlot::Arms) = None;
        assert_eq!(slots.arms, None);
        assert_eq!(slots.head, Some(ArmorSet::Helm));
        assert_eq!(slots.chest, Some(ArmorSet::Mail));
        assert_eq!(slots.legs, Some(ArmorSet::Greaves));
    }

    #[test]
    fn a_slot_borrow_does_not_invent_a_piece() {
        let mut slots = ArmorSlots::default();
        assert_eq!(*slots.piece_mut(ArmorSlot::Chest), None);
        assert_eq!(slots.head, None);
        assert_eq!(slots.chest, None);
        assert_eq!(slots.arms, None);
        assert_eq!(slots.legs, None);
    }
}
