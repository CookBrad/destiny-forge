use std::sync::OnceLock;

use bevy::prelude::*;
use serde::Deserialize;

use crate::core::data_load::load_ron_from_assets_or_embedded;

const GEAR_PATH: &str = "assets/data/gear.ron";
const EMBEDDED_GEAR: &str = include_str!("../../assets/data/gear.ron");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum GearSlot {
    Weapon,
    Head,
    Chest,
    Arms,
    Legs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum GearFamily {
    Sword,
    Spear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum GearSet {
    Slime,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GearPiece {
    pub id: String,
    pub name: String,
    pub slot: GearSlot,
    pub family: Option<GearFamily>,
    pub attack: f32,
    pub defense: f32,
    pub set: Option<GearSet>,
}

#[derive(Resource, Clone, Debug)]
pub struct GearCatalog {
    pieces: Vec<GearPiece>,
}

impl Default for GearCatalog {
    fn default() -> Self {
        Self::load()
    }
}

impl GearCatalog {
    pub fn load() -> Self {
        gear_catalog().clone()
    }

    pub fn piece(&self, id: &str) -> Option<&GearPiece> {
        self.pieces.iter().find(|piece| piece.id == id)
    }

    pub fn name<'a>(&'a self, id: &'a str) -> &'a str {
        self.piece(id)
            .map(|piece| piece.name.as_str())
            .unwrap_or(id)
    }

    pub fn attack(&self, id: &str) -> f32 {
        self.piece(id).map_or(0.0, |piece| piece.attack)
    }

    pub fn defense(&self, id: &str) -> f32 {
        self.piece(id).map_or(0.0, |piece| piece.defense)
    }

    pub fn family(&self, id: &str) -> Option<GearFamily> {
        self.piece(id).and_then(|piece| piece.family)
    }

    pub fn slot(&self, id: &str) -> Option<GearSlot> {
        self.piece(id).map(|piece| piece.slot)
    }

    pub fn set(&self, id: &str) -> Option<GearSet> {
        self.piece(id).and_then(|piece| piece.set)
    }
}

pub(crate) fn gear_catalog() -> &'static GearCatalog {
    static LOADED: OnceLock<GearCatalog> = OnceLock::new();
    LOADED.get_or_init(read_gear_catalog)
}

fn read_gear_catalog() -> GearCatalog {
    match load_ron_from_assets_or_embedded::<Vec<GearPiece>>(GEAR_PATH, EMBEDDED_GEAR, "gear") {
        Some(pieces) if !pieces.is_empty() => GearCatalog { pieces },
        Some(_) => {
            bevy::log::error!("Gear list is empty; weapons and armor have no stats");
            GearCatalog { pieces: Vec::new() }
        }
        None => GearCatalog { pieces: Vec::new() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::WeaponKind;
    use crate::player::{weapon_kind_label, ArmorKind, ArmorSlot};

    #[test]
    fn shipped_rows_keep_weapon_power_family_and_names() {
        let catalog = GearCatalog::load();
        assert_eq!(catalog.attack("rusty_sword"), 10.0);
        assert_eq!(catalog.attack("rusty_spear"), 12.0);
        assert_eq!(catalog.attack("iron_sword"), 14.0);
        assert_eq!(catalog.attack("slime_blade"), 18.0);
        assert_eq!(catalog.family("rusty_spear"), Some(GearFamily::Spear));
        assert_eq!(catalog.family("slime_blade"), Some(GearFamily::Sword));
        assert_eq!(catalog.slot("iron_sword"), Some(GearSlot::Weapon));
        assert_eq!(catalog.set("rusty_sword"), None);
        assert_eq!(catalog.name("rusty_spear"), "Rusty Spear");

        assert_eq!(WeaponKind::RustySword.gear_id(), "rusty_sword");
        assert_eq!(WeaponKind::RustySpear.gear_id(), "rusty_spear");
        assert_eq!(WeaponKind::IronSword.gear_id(), "iron_sword");
        assert_eq!(WeaponKind::SlimeBlade.gear_id(), "slime_blade");
        assert_eq!(WeaponKind::RustySword.base_power(), 10.0);
        assert_eq!(WeaponKind::RustySpear.base_power(), 12.0);
        assert_eq!(WeaponKind::IronSword.base_power(), 14.0);
        assert_eq!(
            WeaponKind::IronSword.family(),
            WeaponKind::RustySword.family()
        );
        assert_ne!(
            WeaponKind::RustySpear.family(),
            WeaponKind::RustySword.family()
        );
        assert_eq!(weapon_kind_label(WeaponKind::SlimeBlade), "Slime Blade");
    }

    #[test]
    fn shipped_rows_keep_armor_defense_slots_and_slime_set() {
        let catalog = GearCatalog::load();
        assert_eq!(catalog.defense("slime_helm"), 2.0);
        assert_eq!(catalog.defense("slime_mail"), 4.0);
        assert_eq!(catalog.defense("slime_gauntlets"), 1.0);
        assert_eq!(catalog.defense("slime_greaves"), 2.0);
        assert_eq!(catalog.slot("slime_helm"), Some(GearSlot::Head));
        assert_eq!(catalog.slot("slime_mail"), Some(GearSlot::Chest));
        assert_eq!(catalog.slot("slime_gauntlets"), Some(GearSlot::Arms));
        assert_eq!(catalog.slot("slime_greaves"), Some(GearSlot::Legs));
        assert_eq!(catalog.family("slime_helm"), None);
        assert_eq!(catalog.set("slime_mail"), Some(GearSet::Slime));
        assert_eq!(catalog.name("slime_gauntlets"), "Slime Gauntlets");

        assert_eq!(ArmorKind::SlimeHelm.gear_id(), "slime_helm");
        assert_eq!(ArmorKind::SlimeMail.gear_id(), "slime_mail");
        assert_eq!(ArmorKind::SlimeGauntlets.gear_id(), "slime_gauntlets");
        assert_eq!(ArmorKind::SlimeGreaves.gear_id(), "slime_greaves");
        assert_eq!(ArmorKind::SlimeHelm.slot(), ArmorSlot::Head);
        assert_eq!(ArmorKind::SlimeMail.slot(), ArmorSlot::Chest);
        assert_eq!(ArmorKind::SlimeGauntlets.slot(), ArmorSlot::Arms);
        assert_eq!(ArmorKind::SlimeGreaves.slot(), ArmorSlot::Legs);
        assert_eq!(ArmorKind::SlimeHelm.defense(), 2.0);
        assert_eq!(ArmorKind::SlimeMail.defense(), 4.0);
        assert_eq!(ArmorKind::SlimeGauntlets.defense(), 1.0);
        assert_eq!(ArmorKind::SlimeGreaves.defense(), 2.0);
        assert_eq!(ArmorKind::SlimeGreaves.label(), "Slime Greaves");
    }

    #[test]
    fn gear_ids_are_unique() {
        let catalog = GearCatalog::load();
        let mut ids: Vec<&str> = catalog
            .pieces
            .iter()
            .map(|piece| piece.id.as_str())
            .collect();
        let before = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), before);
        assert!(before >= 8);
    }
}
