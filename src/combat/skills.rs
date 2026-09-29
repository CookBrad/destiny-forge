use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::weapon::{WeaponFamily, WeaponKind};

pub const SKILL_SLOT_COUNT: usize = 9;
/// Edge length of a skill icon. Each file under `assets/ui/skills/` is one
/// 32×32 image drawn by `tools/draw_skill_icons.py`, not a multi-cell sheet.
const ICON_TILE: f32 = 32.0;

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

    /// Whole-image crop. Charge and Spin used to sample cells out of sheets.
    pub fn icon_rect(&self) -> Rect {
        match self {
            Self::Attack | Self::Block | Self::Charge | Self::Spin => Rect {
                min: Vec2::ZERO,
                max: Vec2::new(ICON_TILE, ICON_TILE),
            },
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
                && Self::key_for_slot(slot).is_some_and(|key| keyboard.just_pressed(key))
        })
    }

    pub fn skill_pressed(
        keyboard: &ButtonInput<KeyCode>,
        bindings: &SkillBindings,
        skill: SkillKind,
    ) -> bool {
        bindings.slots.iter().enumerate().any(|(slot, bound)| {
            bound == &Some(skill)
                && Self::key_for_slot(slot).is_some_and(|key| keyboard.pressed(key))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SkillKind;

    const SKILL_KINDS: [SkillKind; 4] = [
        SkillKind::Attack,
        SkillKind::Block,
        SkillKind::Charge,
        SkillKind::Spin,
    ];

    #[test]
    fn icon_rect_covers_the_full_image_at_the_origin() {
        for skill in SKILL_KINDS {
            let rect = skill.icon_rect();
            assert_eq!(rect.min.x, 0.0, "{skill:?}");
            assert_eq!(rect.min.y, 0.0, "{skill:?}");
            assert_eq!(rect.max.x - rect.min.x, 32.0, "{skill:?}");
            assert_eq!(rect.max.y - rect.min.y, 32.0, "{skill:?}");
        }
    }

    #[test]
    fn skill_icon_files_are_32px_rgba_pngs() {
        for skill in SKILL_KINDS {
            let path = format!(
                "{}/assets/{}",
                env!("CARGO_MANIFEST_DIR"),
                skill.icon_path()
            );
            assert_png_rgba32(&path);
        }
    }

    fn assert_png_rgba32(path: &str) {
        let bytes = std::fs::read(path).unwrap_or_else(|err| panic!("read {path}: {err}"));
        assert!(bytes.len() >= 26, "{path} is too small to be a PNG");
        assert_eq!(
            &bytes[..8],
            &[137, 80, 78, 71, 13, 10, 26, 10],
            "{path} is not a PNG"
        );
        assert_eq!(&bytes[12..16], b"IHDR", "{path} is missing IHDR");
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        let bit_depth = bytes[24];
        let color_type = bytes[25];
        assert_eq!(
            (width, height, bit_depth, color_type),
            (32, 32, 8, 6),
            "{path} is not a 32×32 RGBA8 PNG"
        );
    }
}
