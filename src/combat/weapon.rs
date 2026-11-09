use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gear::{gear_catalog, GearFamily};
use crate::graphics::hunter_blade_tip_reach;

const SWORD_BLADE_REACH: f32 = hunter_blade_tip_reach();
const SPEAR_POKE_REACH: f32 = SWORD_BLADE_REACH * 40.0 / 30.0;
const SPEAR_THRUST_REACH: f32 = SWORD_BLADE_REACH * 48.0 / 32.0;
const SPEAR_LUNGE_REACH: f32 = SWORD_BLADE_REACH * 58.0 / 34.0;

pub const fn spear_special_reach() -> f32 {
    SPEAR_LUNGE_REACH * 64.0 / 58.0
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct EquippedWeapon(pub WeaponKind);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponKind {
    #[default]
    RustySword,
    RustySpear,
    IronSword,
    SlimeBlade,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponFamily {
    Sword,
    Spear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitShape {
    SwordArc,
    SpearThrust,
    SpearLunge,
}

#[derive(Clone, Copy, Debug)]
pub struct ComboStep {
    pub duration: f32,
    pub hit_start: f32,
    pub hit_end: f32,
    pub power_mult: f32,
    pub reach: f32,
    pub shape: HitShape,
    pub chain_start: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct WeaponMoveset {
    pub steps: &'static [ComboStep],
}

const SWORD_COMBO: &[ComboStep] = &[
    ComboStep {
        duration: 0.26,
        hit_start: 0.07,
        hit_end: 0.16,
        power_mult: 1.0,
        reach: SWORD_BLADE_REACH,
        shape: HitShape::SwordArc,
        chain_start: 0.14,
    },
    ComboStep {
        duration: 0.28,
        hit_start: 0.08,
        hit_end: 0.18,
        power_mult: 1.15,
        reach: SWORD_BLADE_REACH,
        shape: HitShape::SwordArc,
        chain_start: 0.16,
    },
    ComboStep {
        duration: 0.36,
        hit_start: 0.1,
        hit_end: 0.24,
        power_mult: 1.4,
        reach: SWORD_BLADE_REACH,
        shape: HitShape::SwordArc,
        chain_start: 0.28,
    },
];

const SPEAR_COMBO: &[ComboStep] = &[
    ComboStep {
        duration: 0.3,
        hit_start: 0.08,
        hit_end: 0.18,
        power_mult: 1.0,
        reach: SPEAR_POKE_REACH,
        shape: HitShape::SpearThrust,
        chain_start: 0.16,
    },
    ComboStep {
        duration: 0.34,
        hit_start: 0.1,
        hit_end: 0.22,
        power_mult: 1.2,
        reach: SPEAR_THRUST_REACH,
        shape: HitShape::SpearThrust,
        chain_start: 0.18,
    },
    ComboStep {
        duration: 0.42,
        hit_start: 0.12,
        hit_end: 0.3,
        power_mult: 1.45,
        reach: SPEAR_LUNGE_REACH,
        shape: HitShape::SpearLunge,
        chain_start: 0.32,
    },
];

impl WeaponKind {
    pub(crate) fn gear_id(self) -> &'static str {
        match self {
            Self::RustySword => "rusty_sword",
            Self::RustySpear => "rusty_spear",
            Self::IronSword => "iron_sword",
            Self::SlimeBlade => "slime_blade",
        }
    }

    pub fn family(self) -> WeaponFamily {
        match gear_catalog().family(self.gear_id()) {
            Some(GearFamily::Spear) => WeaponFamily::Spear,
            Some(GearFamily::Sword) | None => WeaponFamily::Sword,
        }
    }

    pub fn base_power(self) -> f32 {
        gear_catalog().attack(self.gear_id())
    }

    pub fn moveset(self) -> WeaponMoveset {
        match self.family() {
            WeaponFamily::Sword => WeaponMoveset { steps: SWORD_COMBO },
            WeaponFamily::Spear => WeaponMoveset { steps: SPEAR_COMBO },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sword_and_spear_have_distinct_movesets() {
        let sword = WeaponKind::RustySword.moveset();
        let spear = WeaponKind::RustySpear.moveset();
        assert_eq!(sword.steps.len(), 3);
        assert_eq!(spear.steps.len(), 3);
        assert!(matches!(sword.steps[0].shape, HitShape::SwordArc));
        assert!(matches!(spear.steps[0].shape, HitShape::SpearThrust));
        let body_half = crate::graphics::HUNTER_BODY_PX.x * 0.5;
        for step in 0..3 {
            assert!(sword.steps[step].reach > body_half);
            assert!(spear.steps[step].reach > sword.steps[step].reach);
        }
        assert_eq!(
            sword.steps[0].reach,
            crate::graphics::hunter_blade_tip_reach()
        );
    }

    #[test]
    fn slime_blade_keeps_sword_family() {
        assert_eq!(WeaponKind::SlimeBlade.family(), WeaponFamily::Sword);
        assert_eq!(WeaponKind::SlimeBlade.base_power(), 18.0);
    }

    #[test]
    fn a_missing_catalog_family_maps_to_sword() {
        assert_eq!(
            crate::gear::GearCatalog::load().family("no_such_weapon"),
            None
        );
        assert_eq!(
            match crate::gear::GearCatalog::load().family("no_such_weapon") {
                Some(crate::gear::GearFamily::Spear) => WeaponFamily::Spear,
                Some(crate::gear::GearFamily::Sword) | None => WeaponFamily::Sword,
            },
            WeaponFamily::Sword
        );
    }
}
