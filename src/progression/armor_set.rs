use crate::items::{ArmorId, ArmorSlot};

#[derive(Clone, Copy, Debug)]
pub struct ArmorSetBonus {
    pub pieces_required: u8,
    pub description: &'static str,
    pub carve_speed_multiplier: f32,
    pub knockback_resistance: f32,
}

pub struct SlimeSet;

impl SlimeSet {
    pub const PIECES: [ArmorId; 4] = [
        ArmorId::SlimeHelm,
        ArmorId::SlimeMail,
        ArmorId::SlimeGauntlets,
        ArmorId::SlimeGreaves,
    ];

    pub const BONUSES: [ArmorSetBonus; 2] = [
        ArmorSetBonus {
            pieces_required: 2,
            description: "+10% carve speed",
            carve_speed_multiplier: 1.1,
            knockback_resistance: 0.0,
        },
        ArmorSetBonus {
            pieces_required: 4,
            description: "Reduced knockback",
            carve_speed_multiplier: 1.1,
            knockback_resistance: 0.35,
        },
    ];

    pub fn equipped_piece_count(equipped: &[Option<ArmorId>]) -> u8 {
        equipped
            .iter()
            .filter(|piece| {
                piece.is_some_and(|id| Self::PIECES.contains(&id))
            })
            .count() as u8
    }

    pub fn active_bonuses(equipped: &[Option<ArmorId>]) -> Vec<ArmorSetBonus> {
        let count = Self::equipped_piece_count(equipped);
        Self::BONUSES
            .iter()
            .copied()
            .filter(|bonus| count >= bonus.pieces_required)
            .collect()
    }

    pub fn total_defense_bonus(equipped: &[Option<ArmorId>]) -> f32 {
        equipped
            .iter()
            .filter_map(|piece| piece.map(|id| id.defense()))
            .sum()
    }

    pub fn slot_index(slot: ArmorSlot) -> usize {
        match slot {
            ArmorSlot::Head => 0,
            ArmorSlot::Chest => 1,
            ArmorSlot::Arms => 2,
            ArmorSlot::Legs => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_piece_bonus_unlocks_at_two_slime_pieces() {
        let equipped = [
            Some(ArmorId::SlimeHelm),
            Some(ArmorId::SlimeMail),
            None,
            None,
        ];
        let bonuses = SlimeSet::active_bonuses(&equipped);
        assert_eq!(bonuses.len(), 1);
        assert!(bonuses[0].carve_speed_multiplier > 1.0);
    }
}