use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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

/// How a combo step produces its hit volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitShape {
    /// Vertical arc (sword).
    SwordArc,
    /// Forward poke (spear).
    SpearThrust,
    /// Long commit poke (spear finisher / lunge).
    SpearLunge,
}

#[derive(Clone, Copy, Debug)]
pub struct ComboStep {
    pub duration: f32,
    pub hit_start: f32,
    pub hit_end: f32,
    /// Multiplier on weapon base power for this step.
    pub power_mult: f32,
    /// Forward reach in world pixels (spear); swords ignore for arc geometry.
    pub reach: f32,
    pub shape: HitShape,
    /// Elapsed time when input can queue the next combo step.
    pub chain_start: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct WeaponMoveset {
    pub steps: &'static [ComboStep],
}

/// Legacy single-swing stats used by non-combo callers; prefer `moveset` + `base_power`.
#[derive(Clone, Copy, Debug)]
pub struct WeaponStats {
    pub attack_power: f32,
    pub reach: f32,
    pub swing_secs: f32,
    pub hit_start: f32,
    pub hit_end: f32,
}

// --- Sword combo: fast multi-hit ---
const SWORD_COMBO: &[ComboStep] = &[
    ComboStep {
        duration: 0.26,
        hit_start: 0.07,
        hit_end: 0.16,
        power_mult: 1.0,
        reach: 120.0,
        shape: HitShape::SwordArc,
        chain_start: 0.14,
    },
    ComboStep {
        duration: 0.28,
        hit_start: 0.08,
        hit_end: 0.18,
        power_mult: 1.15,
        reach: 128.0,
        shape: HitShape::SwordArc,
        chain_start: 0.16,
    },
    ComboStep {
        duration: 0.36,
        hit_start: 0.1,
        hit_end: 0.24,
        power_mult: 1.4,
        reach: 136.0,
        shape: HitShape::SwordArc,
        chain_start: 0.28,
    },
];

// --- Spear combo: poke → thrust → lunge ---
const SPEAR_COMBO: &[ComboStep] = &[
    ComboStep {
        duration: 0.3,
        hit_start: 0.08,
        hit_end: 0.18,
        power_mult: 1.0,
        reach: 160.0,
        shape: HitShape::SpearThrust,
        chain_start: 0.16,
    },
    ComboStep {
        duration: 0.34,
        hit_start: 0.1,
        hit_end: 0.22,
        power_mult: 1.2,
        reach: 192.0,
        shape: HitShape::SpearThrust,
        chain_start: 0.18,
    },
    ComboStep {
        duration: 0.42,
        hit_start: 0.12,
        hit_end: 0.3,
        power_mult: 1.45,
        reach: 232.0,
        shape: HitShape::SpearLunge,
        chain_start: 0.32,
    },
];

impl WeaponKind {
    pub fn family(self) -> WeaponFamily {
        match self {
            Self::RustySword | Self::IronSword | Self::SlimeBlade => WeaponFamily::Sword,
            Self::RustySpear => WeaponFamily::Spear,
        }
    }

    pub fn is_sword(self) -> bool {
        matches!(self.family(), WeaponFamily::Sword)
    }

    pub fn is_spear(self) -> bool {
        matches!(self.family(), WeaponFamily::Spear)
    }

    pub fn base_power(self) -> f32 {
        match self {
            Self::RustySword => 10.0,
            Self::RustySpear => 12.0,
            Self::IronSword => 14.0,
            Self::SlimeBlade => 18.0,
        }
    }

    pub fn moveset(self) -> WeaponMoveset {
        match self.family() {
            WeaponFamily::Sword => WeaponMoveset { steps: SWORD_COMBO },
            WeaponFamily::Spear => WeaponMoveset { steps: SPEAR_COMBO },
        }
    }

    pub fn stats(self) -> WeaponStats {
        let step = self.moveset().steps[0];
        WeaponStats {
            attack_power: self.base_power(),
            reach: step.reach,
            swing_secs: step.duration,
            hit_start: step.hit_start,
            hit_end: step.hit_end,
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
        assert!(spear.steps[2].reach > sword.steps[2].reach);
    }

    #[test]
    fn slime_blade_keeps_sword_family() {
        assert!(WeaponKind::SlimeBlade.is_sword());
        assert_eq!(WeaponKind::SlimeBlade.base_power(), 18.0);
    }
}
