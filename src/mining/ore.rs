use crate::core::{PlayerProfile, ToolEnergy};
use crate::items::{Inventory, MaterialId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OreKind {
    Copper,
    Iron,
}

impl OreKind {
    pub fn hits_to_break(self) -> u32 {
        match self {
            Self::Copper => 2,
            Self::Iron => 4,
        }
    }

    pub fn energy_reserve_required(self) -> f32 {
        match self {
            Self::Copper => 4.0,
            Self::Iron => 8.0,
        }
    }

    pub fn hardness(self) -> u8 {
        match self {
            Self::Copper => 1,
            Self::Iron => 2,
        }
    }

    pub fn ore_material(self) -> MaterialId {
        match self {
            Self::Copper => MaterialId::CopperOre,
            Self::Iron => MaterialId::IronOre,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Copper => "copper",
            Self::Iron => "iron",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PickaxeTier {
    Tier1,
    Tier2,
}

impl PickaxeTier {
    pub fn mining_power(self) -> u8 {
        match self {
            Self::Tier1 => 1,
            Self::Tier2 => 2,
        }
    }

    pub fn energy_cost(self) -> f32 {
        match self {
            Self::Tier1 => 4.0,
            Self::Tier2 => 3.0,
        }
    }

    pub fn can_mine(self, ore: OreKind) -> bool {
        self.mining_power() >= ore.hardness()
    }

    pub fn best_owned(inventory: &Inventory) -> Option<Self> {
        if inventory.count(MaterialId::PickaxeTier2) > 0 {
            Some(Self::Tier2)
        } else if inventory.count(MaterialId::PickaxeTier1) > 0 {
            Some(Self::Tier1)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwingOutcome {
    NoPickaxe,
    TooHard,
    NotEnoughEnergy,
    Cracked { hits_taken: u32 },
    BrokeAndDroppedOre,
    NoRoomForOre,
}

pub fn reason_swing_cannot_land(
    ore: OreKind,
    node_hits: u32,
    pickaxe: Option<PickaxeTier>,
    energy: &ToolEnergy,
    inventory: &Inventory,
) -> Option<SwingOutcome> {
    let Some(pickaxe) = pickaxe else {
        return Some(SwingOutcome::NoPickaxe);
    };
    if !pickaxe.can_mine(ore) {
        return Some(SwingOutcome::TooHard);
    }
    let swing_breaks_node = node_hits + 1 >= ore.hits_to_break();
    if swing_breaks_node && !inventory.has_room_for(ore.ore_material()) {
        return Some(SwingOutcome::NoRoomForOre);
    }
    let below_energy_reserve = energy.current + f32::EPSILON < ore.energy_reserve_required();
    let cannot_pay_swing = energy.current + f32::EPSILON < pickaxe.energy_cost();
    if below_energy_reserve || cannot_pay_swing {
        return Some(SwingOutcome::NotEnoughEnergy);
    }
    None
}

pub fn swing_pickaxe(
    ore: OreKind,
    node_hits: &mut u32,
    pickaxe: Option<PickaxeTier>,
    energy: &mut ToolEnergy,
    inventory: &mut Inventory,
) -> SwingOutcome {
    if let Some(blocked) = reason_swing_cannot_land(ore, *node_hits, pickaxe, energy, inventory) {
        return blocked;
    }
    let Some(pickaxe) = pickaxe else {
        return SwingOutcome::NoPickaxe;
    };
    if !energy.try_spend(pickaxe.energy_cost()) {
        return SwingOutcome::NotEnoughEnergy;
    }
    *node_hits += 1;
    if *node_hits >= ore.hits_to_break() {
        inventory.try_add(ore.ore_material(), 1);
        SwingOutcome::BrokeAndDroppedOre
    } else {
        SwingOutcome::Cracked {
            hits_taken: *node_hits,
        }
    }
}

pub fn mark_node_depleted(profile: &mut PlayerProfile, id: u16) {
    if !profile.depleted_ore_node_ids.contains(&id) {
        profile.depleted_ore_node_ids.push(id);
    }
}

pub fn respawn_ore_nodes_on_sleep(profile: &mut PlayerProfile) {
    profile.depleted_ore_node_ids.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_tool_energy() -> ToolEnergy {
        ToolEnergy::default()
    }

    #[test]
    fn tier1_mines_copper_only_and_tier2_mines_both() {
        assert!(PickaxeTier::Tier1.can_mine(OreKind::Copper));
        assert!(!PickaxeTier::Tier1.can_mine(OreKind::Iron));
        assert!(PickaxeTier::Tier2.can_mine(OreKind::Copper));
        assert!(PickaxeTier::Tier2.can_mine(OreKind::Iron));
    }

    #[test]
    fn ore_hits_energy_and_pickaxe_costs_match_design() {
        assert_eq!(OreKind::Copper.hits_to_break(), 2);
        assert_eq!(OreKind::Copper.energy_reserve_required(), 4.0);
        assert_eq!(OreKind::Iron.hits_to_break(), 4);
        assert_eq!(OreKind::Iron.energy_reserve_required(), 8.0);
        assert_eq!(PickaxeTier::Tier1.energy_cost(), 4.0);
        assert_eq!(PickaxeTier::Tier2.energy_cost(), 3.0);
    }

    #[test]
    fn copper_breaks_in_two_tier1_swings() {
        let mut energy = full_tool_energy();
        let mut inventory = Inventory::default();
        let mut node_hits = 0;
        let tier1 = Some(PickaxeTier::Tier1);
        assert_eq!(
            swing_pickaxe(
                OreKind::Copper,
                &mut node_hits,
                tier1,
                &mut energy,
                &mut inventory
            ),
            SwingOutcome::Cracked { hits_taken: 1 }
        );
        assert_eq!(
            swing_pickaxe(
                OreKind::Copper,
                &mut node_hits,
                tier1,
                &mut energy,
                &mut inventory
            ),
            SwingOutcome::BrokeAndDroppedOre
        );
        assert_eq!(inventory.count(MaterialId::CopperOre), 1);
        assert_eq!(energy.current, energy.max - 8.0);
    }

    #[test]
    fn tier1_on_iron_is_too_hard_and_spends_nothing() {
        let mut energy = full_tool_energy();
        let mut inventory = Inventory::default();
        let mut node_hits = 0;
        assert_eq!(
            swing_pickaxe(
                OreKind::Iron,
                &mut node_hits,
                Some(PickaxeTier::Tier1),
                &mut energy,
                &mut inventory
            ),
            SwingOutcome::TooHard
        );
        assert_eq!(node_hits, 0);
        assert_eq!(energy.current, energy.max);
    }

    #[test]
    fn iron_breaks_in_four_tier2_swings() {
        let mut energy = full_tool_energy();
        let mut inventory = Inventory::default();
        let mut node_hits = 0;
        let tier2 = Some(PickaxeTier::Tier2);
        for expected_hits in 1..4 {
            assert_eq!(
                swing_pickaxe(
                    OreKind::Iron,
                    &mut node_hits,
                    tier2,
                    &mut energy,
                    &mut inventory
                ),
                SwingOutcome::Cracked {
                    hits_taken: expected_hits
                }
            );
        }
        assert_eq!(
            swing_pickaxe(
                OreKind::Iron,
                &mut node_hits,
                tier2,
                &mut energy,
                &mut inventory
            ),
            SwingOutcome::BrokeAndDroppedOre
        );
        assert_eq!(inventory.count(MaterialId::IronOre), 1);
        assert_eq!(energy.current, energy.max - 12.0);
    }

    #[test]
    fn iron_needs_eight_energy_in_reserve() {
        let mut energy = ToolEnergy::from_saved(7.0, 100.0);
        let mut inventory = Inventory::default();
        let mut node_hits = 0;
        assert_eq!(
            swing_pickaxe(
                OreKind::Iron,
                &mut node_hits,
                Some(PickaxeTier::Tier2),
                &mut energy,
                &mut inventory
            ),
            SwingOutcome::NotEnoughEnergy
        );
        assert_eq!(energy.current, 7.0);
        assert_eq!(node_hits, 0);
    }

    #[test]
    fn best_owned_pickaxe_prefers_tier2() {
        let mut inventory = Inventory::default();
        assert_eq!(PickaxeTier::best_owned(&inventory), None);
        inventory.try_add(MaterialId::PickaxeTier1, 1);
        assert_eq!(
            PickaxeTier::best_owned(&inventory),
            Some(PickaxeTier::Tier1)
        );
        inventory.try_add(MaterialId::PickaxeTier2, 1);
        assert_eq!(
            PickaxeTier::best_owned(&inventory),
            Some(PickaxeTier::Tier2)
        );
    }

    #[test]
    fn depleted_ids_persist_until_sleep() {
        let mut profile = PlayerProfile::default();
        mark_node_depleted(&mut profile, 3);
        mark_node_depleted(&mut profile, 3);
        mark_node_depleted(&mut profile, 9);
        assert_eq!(profile.depleted_ore_node_ids, vec![3, 9]);

        let saved_ron = ron::to_string(&profile).expect("serialize");
        let loaded: PlayerProfile = ron::from_str(&saved_ron).expect("deserialize");
        assert_eq!(loaded.depleted_ore_node_ids, vec![3, 9]);

        respawn_ore_nodes_on_sleep(&mut profile);
        assert!(profile.depleted_ore_node_ids.is_empty());
    }
}
