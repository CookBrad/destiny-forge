//! Pure mining rules: hardness vs pickaxe power, ore drops.

use crate::items::MaterialId;

/// Soft surface iron node hardness (basic pickaxe = 1 can break).
pub const SOFT_IRON_HARDNESS: u32 = 1;

/// Default ore yield from a soft iron node.
pub const SOFT_IRON_ORE_AMOUNT: u32 = 2;

/// Result of attempting to mine a node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MineResult {
    /// Node broken; grant materials.
    Broke { drops: Vec<(MaterialId, u32)> },
    /// Pickaxe too weak for this hardness.
    TooHard { required: u32, power: u32 },
    /// No pickaxe / power 0.
    NoPickaxe,
}

/// Can this pickaxe power break a node of the given hardness?
pub fn can_break_node(pickaxe_power: u32, hardness: u32) -> bool {
    pickaxe_power > 0 && pickaxe_power >= hardness
}

/// Resolve a mine swing against a node.
///
/// Soft iron nodes drop [`MaterialId::IronOre`]. Harder nodes need higher power.
pub fn try_mine_node(pickaxe_power: u32, hardness: u32) -> MineResult {
    if pickaxe_power == 0 {
        return MineResult::NoPickaxe;
    }
    if !can_break_node(pickaxe_power, hardness) {
        return MineResult::TooHard {
            required: hardness,
            power: pickaxe_power,
        };
    }
    MineResult::Broke {
        drops: soft_iron_drops(hardness),
    }
}

fn soft_iron_drops(hardness: u32) -> Vec<(MaterialId, u32)> {
    // Soft (1): 2 ore. Slightly harder still yields ore in first slice.
    let amount = if hardness <= SOFT_IRON_HARDNESS {
        SOFT_IRON_ORE_AMOUNT
    } else {
        SOFT_IRON_ORE_AMOUNT + 1
    };
    vec![(MaterialId::IronOre, amount)]
}

/// Energy cost for one pickaxe swing (mirrors MaterialId::Pickaxe).
pub fn pickaxe_energy_cost() -> f32 {
    MaterialId::Pickaxe.energy_cost()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soft_node_yields_ore_with_basic_pickaxe() {
        let result = try_mine_node(1, SOFT_IRON_HARDNESS);
        assert_eq!(
            result,
            MineResult::Broke {
                drops: vec![(MaterialId::IronOre, SOFT_IRON_ORE_AMOUNT)]
            }
        );
    }

    #[test]
    fn weak_pickaxe_cannot_break_hard_node() {
        let result = try_mine_node(1, 3);
        assert_eq!(
            result,
            MineResult::TooHard {
                required: 3,
                power: 1
            }
        );
    }

    #[test]
    fn zero_power_is_no_pickaxe() {
        assert_eq!(try_mine_node(0, 1), MineResult::NoPickaxe);
        assert!(!can_break_node(0, 1));
    }

    #[test]
    fn can_break_requires_power_gte_hardness() {
        assert!(can_break_node(1, 1));
        assert!(can_break_node(2, 1));
        assert!(!can_break_node(1, 2));
    }
}
