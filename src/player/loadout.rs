use bevy::prelude::*;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize, Serializer};

use crate::combat::{EquippedWeapon, WeaponKind};
use crate::items::{FoodBuff, Inventory, MaterialId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArmorPiece {
    Helm,
    Mail,
    Gauntlets,
    Greaves,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmorSet {
    Slime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmorSlot {
    Helm,
    Mail,
    Gauntlets,
    Greaves,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArmorSlots {
    pub helm: Option<ArmorPiece>,
    pub mail: Option<ArmorPiece>,
    pub gauntlets: Option<ArmorPiece>,
    pub greaves: Option<ArmorPiece>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GearStash {
    pub weapons: Vec<WeaponKind>,
    pub armor: Vec<ArmorPiece>,
}

#[derive(Resource, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Loadout {
    pub weapon: WeaponKind,
    pub armor: ArmorSlots,
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

impl ArmorPiece {
    pub fn set(self) -> ArmorSet {
        ArmorSet::Slime
    }

    pub fn slot(self) -> ArmorSlot {
        match self {
            Self::Helm => ArmorSlot::Helm,
            Self::Mail => ArmorSlot::Mail,
            Self::Gauntlets => ArmorSlot::Gauntlets,
            Self::Greaves => ArmorSlot::Greaves,
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

impl Serialize for ArmorPiece {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        StoredArmorPiece::from(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ArmorPiece {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        StoredArmorPiece::deserialize(deserializer).map(Self::from)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum StoredArmorPiece {
    SlimeHelm,
    SlimeMail,
    SlimeGauntlets,
    SlimeGreaves,
}

impl From<ArmorPiece> for StoredArmorPiece {
    fn from(piece: ArmorPiece) -> Self {
        match piece {
            ArmorPiece::Helm => Self::SlimeHelm,
            ArmorPiece::Mail => Self::SlimeMail,
            ArmorPiece::Gauntlets => Self::SlimeGauntlets,
            ArmorPiece::Greaves => Self::SlimeGreaves,
        }
    }
}

impl From<StoredArmorPiece> for ArmorPiece {
    fn from(stored: StoredArmorPiece) -> Self {
        match stored {
            StoredArmorPiece::SlimeHelm => Self::Helm,
            StoredArmorPiece::SlimeMail => Self::Mail,
            StoredArmorPiece::SlimeGauntlets => Self::Gauntlets,
            StoredArmorPiece::SlimeGreaves => Self::Greaves,
        }
    }
}

impl ArmorSlots {
    pub fn set(&mut self, piece: ArmorPiece) {
        match piece.slot() {
            ArmorSlot::Helm => self.helm = Some(piece),
            ArmorSlot::Mail => self.mail = Some(piece),
            ArmorSlot::Gauntlets => self.gauntlets = Some(piece),
            ArmorSlot::Greaves => self.greaves = Some(piece),
        }
    }

    fn piece_mut(&mut self, slot: ArmorSlot) -> &mut Option<ArmorPiece> {
        match slot {
            ArmorSlot::Helm => &mut self.helm,
            ArmorSlot::Mail => &mut self.mail,
            ArmorSlot::Gauntlets => &mut self.gauntlets,
            ArmorSlot::Greaves => &mut self.greaves,
        }
    }
}

impl Serialize for ArmorSlots {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        StoredArmorSlots {
            head: self.helm,
            chest: self.mail,
            arms: self.gauntlets,
            legs: self.greaves,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ArmorSlots {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let stored = StoredArmorSlots::deserialize(deserializer)?;
        Ok(Self {
            helm: stored.head,
            mail: stored.chest,
            gauntlets: stored.arms,
            greaves: stored.legs,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct StoredArmorSlots {
    head: Option<ArmorPiece>,
    chest: Option<ArmorPiece>,
    arms: Option<ArmorPiece>,
    legs: Option<ArmorPiece>,
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

    pub fn equip_forged_armor(&mut self, piece: ArmorPiece) {
        let slot = piece.slot();
        if let Some(previous) = *self.armor.piece_mut(slot) {
            if previous != piece {
                self.store_armor(previous);
            }
        }
        self.take_armor_from_stash(piece);
        self.armor.set(piece);
    }

    pub fn swap_to_stashed_armor(&mut self, piece: ArmorPiece) -> bool {
        if !self.stash.armor.contains(&piece) {
            return false;
        }
        self.equip_forged_armor(piece);
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

    fn store_armor(&mut self, piece: ArmorPiece) {
        if !self.stash.armor.contains(&piece) {
            self.stash.armor.push(piece);
        }
    }

    fn take_armor_from_stash(&mut self, piece: ArmorPiece) {
        self.stash.armor.retain(|stored| *stored != piece);
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
        if let Some(piece) = self.armor.helm {
            total += piece.defense();
        }
        if let Some(piece) = self.armor.mail {
            total += piece.defense();
        }
        if let Some(piece) = self.armor.gauntlets {
            total += piece.defense();
        }
        if let Some(piece) = self.armor.greaves {
            total += piece.defense();
        }
        total
    }

    pub fn slime_set_pieces(&self) -> u32 {
        [
            self.armor.helm,
            self.armor.mail,
            self.armor.gauntlets,
            self.armor.greaves,
        ]
        .into_iter()
        .filter(|piece| piece.is_some_and(|piece| piece.set() == ArmorSet::Slime))
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

    fn full_slime() -> Loadout {
        let mut loadout = Loadout::default();
        loadout.armor.helm = Some(ArmorPiece::Helm);
        loadout.armor.mail = Some(ArmorPiece::Mail);
        loadout.armor.gauntlets = Some(ArmorPiece::Gauntlets);
        loadout.armor.greaves = Some(ArmorPiece::Greaves);
        loadout
    }

    #[test]
    fn two_piece_gives_carve_and_special_cd() {
        let mut loadout = Loadout::default();
        loadout.armor.helm = Some(ArmorPiece::Helm);
        loadout.armor.mail = Some(ArmorPiece::Mail);
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

        app.add_systems(Update, expire_food_buff_after_hunt);
        app.update();

        assert_eq!(app.world().resource::<Loadout>().food_buff, None);
    }

    #[test]
    fn helm_is_in_the_slime_set() {
        assert_eq!(ArmorPiece::Helm.set(), ArmorSet::Slime);
    }

    #[test]
    fn mail_is_in_the_slime_set() {
        assert_eq!(ArmorPiece::Mail.set(), ArmorSet::Slime);
    }

    #[test]
    fn gauntlets_are_in_the_slime_set() {
        assert_eq!(ArmorPiece::Gauntlets.set(), ArmorSet::Slime);
    }

    #[test]
    fn greaves_are_in_the_slime_set() {
        assert_eq!(ArmorPiece::Greaves.set(), ArmorSet::Slime);
    }

    #[test]
    fn old_loadout_without_food_buff_field_loads() {
        let old = "(weapon: RustySword, armor: (head: None, chest: None, arms: None, legs: None))";
        let loadout: Loadout = ron::from_str(old).unwrap();
        assert_eq!(loadout.food_buff, None);
    }

    #[test]
    fn saved_slime_armor_names_load_into_the_piece_slots() {
        let old = "(weapon: RustySword, armor: (head: Some(SlimeHelm), chest: Some(SlimeMail), arms: Some(SlimeGauntlets), legs: Some(SlimeGreaves)), stash: (weapons: [], armor: [SlimeMail]))";
        let loadout: Loadout = ron::from_str(old).unwrap();
        assert_eq!(loadout.armor.helm, Some(ArmorPiece::Helm));
        assert_eq!(loadout.armor.mail, Some(ArmorPiece::Mail));
        assert_eq!(loadout.armor.gauntlets, Some(ArmorPiece::Gauntlets));
        assert_eq!(loadout.armor.greaves, Some(ArmorPiece::Greaves));
        assert_eq!(loadout.stash.armor, vec![ArmorPiece::Mail]);
        let saved = ron::to_string(&loadout).unwrap();
        assert_eq!(
            saved,
            "(weapon:RustySword,armor:(head:Some(SlimeHelm),chest:Some(SlimeMail),arms:Some(SlimeGauntlets),legs:Some(SlimeGreaves)),stash:(weapons:[],armor:[SlimeMail]),food_buff:None)"
        );
    }

    #[test]
    fn forged_armor_stashes_previous_piece_in_slot() {
        let mut loadout = Loadout::default();
        loadout.equip_forged_armor(ArmorPiece::Helm);
        assert_eq!(loadout.armor.helm, Some(ArmorPiece::Helm));
        assert!(loadout.stash.armor.is_empty());
        assert_eq!(loadout.armor.mail, None);
        assert_eq!(loadout.armor.gauntlets, None);
        assert_eq!(loadout.armor.greaves, None);
    }
}
