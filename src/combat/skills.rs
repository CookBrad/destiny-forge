use bevy::prelude::*;

pub const SKILL_SLOT_COUNT: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

    pub fn abbrev(&self) -> &'static str {
        match self {
            Self::Attack => "ATK",
            Self::Block => "BLK",
            Self::Charge => "CHG",
            Self::Spin => "SPN",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Attack => Color::srgb(0.95, 0.42, 0.32),
            Self::Block => Color::srgb(0.38, 0.62, 0.95),
            Self::Charge => Color::srgb(0.95, 0.82, 0.28),
            Self::Spin => Color::srgb(0.72, 0.45, 0.95),
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