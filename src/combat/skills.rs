use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::weapon::{WeaponFamily, WeaponKind};

pub const SKILL_SLOT_COUNT: usize = 9;
const ICON_TILE: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillKind {
    Attack,
    Block,
    Charge,
    Spin,
}

impl SkillKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Attack => "Attack",
            Self::Block => "Block",
            Self::Charge => "Charge",
            Self::Spin => "Spin",
        }
    }

    /// Label adjusted for equipped weapon (Spin → Thrust on spears).
    pub fn label_for_weapon(&self, weapon: WeaponKind) -> &'static str {
        match (self, weapon.family()) {
            (Self::Spin, WeaponFamily::Spear) => "Thrust",
            _ => self.label(),
        }
    }

    pub fn icon_path(&self) -> &'static str {
        match self {
            Self::Attack => "ui/skills/short_wep.png",
            Self::Block => "ui/skills/shield.png",
            Self::Charge => "ui/skills/boot.png",
            Self::Spin => "ui/skills/wand.png",
        }
    }

    pub fn icon_rect(&self) -> Rect {
        let (x, y) = match self {
            Self::Attack => (0.0, 0.0),
            Self::Block => (0.0, 0.0),
            Self::Charge => (16.0, 0.0),
            Self::Spin => (32.0, 16.0),
        };
        Rect {
            min: Vec2::new(x, y),
            max: Vec2::new(x + ICON_TILE, y + ICON_TILE),
        }
    }
}

#[derive(Resource)]
pub struct SkillIconAssets {
    pub attack: Handle<Image>,
    pub block: Handle<Image>,
    pub charge: Handle<Image>,
    pub spin: Handle<Image>,
}

impl SkillIconAssets {
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            attack: asset_server.load(SkillKind::Attack.icon_path()),
            block: asset_server.load(SkillKind::Block.icon_path()),
            charge: asset_server.load(SkillKind::Charge.icon_path()),
            spin: asset_server.load(SkillKind::Spin.icon_path()),
        }
    }

    pub fn handle_for(&self, skill: SkillKind) -> Handle<Image> {
        match skill {
            SkillKind::Attack => self.attack.clone(),
            SkillKind::Block => self.block.clone(),
            SkillKind::Charge => self.charge.clone(),
            SkillKind::Spin => self.spin.clone(),
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct SkillBindings {
    pub slots: [Option<SkillKind>; SKILL_SLOT_COUNT],
}

impl Default for SkillBindings {
    fn default() -> Self {
        let mut slots = [None; SKILL_SLOT_COUNT];
        slots[0] = Some(SkillKind::Attack);
        slots[1] = Some(SkillKind::Block);
        slots[2] = Some(SkillKind::Charge);
        slots[3] = Some(SkillKind::Spin);
        Self { slots }
    }
}

impl SkillBindings {
    pub fn key_for_slot(slot: usize) -> Option<KeyCode> {
        match slot {
            0 => Some(KeyCode::Digit1),
            1 => Some(KeyCode::Digit2),
            2 => Some(KeyCode::Digit3),
            3 => Some(KeyCode::Digit4),
            4 => Some(KeyCode::Digit5),
            5 => Some(KeyCode::Digit6),
            6 => Some(KeyCode::Digit7),
            7 => Some(KeyCode::Digit8),
            8 => Some(KeyCode::Digit9),
            _ => None,
        }
    }

    pub fn swap_slots(&mut self, a: usize, b: usize) {
        if a < SKILL_SLOT_COUNT && b < SKILL_SLOT_COUNT && a != b {
            self.slots.swap(a, b);
        }
    }

    pub fn skill_just_pressed(
        keyboard: &ButtonInput<KeyCode>,
        bindings: &SkillBindings,
        skill: SkillKind,
    ) -> bool {
        bindings.slots.iter().enumerate().any(|(slot, bound)| {
            bound == &Some(skill)
                && Self::key_for_slot(slot)
                    .is_some_and(|key| keyboard.just_pressed(key))
        })
    }

    pub fn skill_pressed(
        keyboard: &ButtonInput<KeyCode>,
        bindings: &SkillBindings,
        skill: SkillKind,
    ) -> bool {
        bindings.slots.iter().enumerate().any(|(slot, bound)| {
            bound == &Some(skill)
                && Self::key_for_slot(slot)
                    .is_some_and(|key| keyboard.pressed(key))
        })
    }
}