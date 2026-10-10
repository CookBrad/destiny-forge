//! Pure mining rules: ore kinds, pickaxe tiers, swing outcomes. No ECS here.

use crate::core::PlayerProfile;
use crate::items::{Inventory, MaterialId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OreKind {
    Copper,
    Iron,
}

impl OreKind {
    /// Pickaxe swings to break one node.
    pub fn hits_required(self) -> u32 {
        match self {
            Self::Copper => 2,
            Self::Iron => 4,
        }
    }

    /// Node energy rating: tool energy you must have in reserve to work this node.
    /// Each swing then costs the pickaxe's `energy_cost`.
    pub fn energy_required(self) -> f32 {
        match self {
            Self::Copper => 4.0,
            Self::Iron => 8.0,
        }
    }

    /// Hardness gate: pickaxe power must be at least this.
    pub fn hardness(self) -> u8 {
        match self {
            Self::Copper => 1,
            Self::Iron => 2,
        }
    }

    pub fn ore(self) -> MaterialId {
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
    /// Starter grant. Copper only.
    Tier1,
    /// Forged. Copper and iron.
    Tier2,
}

impl PickaxeTier {
    pub fn power(self) -> u8 {
        match self {
            Self::Tier1 => 1,
            Self::Tier2 => 2,
        }
    }

    /// Tool energy per swing.
    pub fn energy_cost(self) -> f32 {
        match self {
            Self::Tier1 => 4.0,
            Self::Tier2 => 3.0,
        }
    }

    pub fn can_mine(self, ore: OreKind) -> bool {
        self.power() >= ore.hardness()
    }

    /// Best pickaxe the player owns (T2 beats T1).
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
    /// Node took a hit; `hits` so far.
    Hit { hits: u32 },
    /// Node broke and dropped one ore.
    Broke,
    /// Node broke but the inventory is full; node stays at its last hit.
    InventoryFull,
}

/// Apply one pickaxe swing to a node. Mutates energy, hits, and inventory only on success.
pub fn swing_pickaxe(
    ore: OreKind,
    hits: &mut u32,
    pickaxe: Option<PickaxeTier>,
    energy: &mut crate::core::ToolEnergy,
    inventory: &mut Inventory,
) -> SwingOutcome {
    let Some(pickaxe) = pickaxe else {
        return SwingOutcome::NoPickaxe;
    };
    if !pickaxe.can_mine(ore) {
        return SwingOutcome::TooHard;
    }
    let breaks = *hits + 1 >= ore.hits_required();
    if breaks && !inventory.has_room_for(ore.ore()) {
        return SwingOutcome::InventoryFull;
    }
    if energy.current + f32::EPSILON < ore.energy_required() || !energy.try_spend(pickaxe.energy_cost())
    {
        return SwingOutcome::NotEnoughEnergy;
    }
    *hits += 1;
    if breaks {
        inventory.try_add(ore.ore(), 1);
        SwingOutcome::Broke
    } else {
        SwingOutcome::Hit { hits: *hits }
    }
}

pub fn mark_node_depleted(profile: &mut PlayerProfile, id: u16) {
    if !profile.depleted_ore_nodes.contains(&id) {
        profile.depleted_ore_nodes.push(id);
    }
}

/// Sleep respawns every mine node.
pub fn respawn_ore_nodes_on_sleep(profile: &mut PlayerProfile) {
    profile.depleted_ore_nodes.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ToolEnergy;

    fn full_energy() -> ToolEnergy {
        ToolEnergy::default()
    }

    #[test]
    fn tiers_gate_hardness() {
        assert!(PickaxeTier::Tier1.can_mine(OreKind::Copper));
        assert!(!PickaxeTier::Tier1.can_mine(OreKind::Iron));
        assert!(PickaxeTier::Tier2.can_mine(OreKind::Copper));
        assert!(PickaxeTier::Tier2.can_mine(OreKind::Iron));
    }

    #[test]
    fn locked_costs() {
        assert_eq!(OreKind::Copper.hits_required(), 2);
        assert_eq!(OreKind::Copper.energy_required(), 4.0);
        assert_eq!(OreKind::Iron.hits_required(), 4);
        assert_eq!(OreKind::Iron.energy_required(), 8.0);
        assert_eq!(PickaxeTier::Tier1.energy_cost(), 4.0);
        assert_eq!(PickaxeTier::Tier2.energy_cost(), 3.0);
    }

    #[test]
    fn copper_breaks_in_two_t1_swings() {
        let mut energy = full_energy();
        let mut inventory = Inventory::default();
        let mut hits = 0;
        let t1 = Some(PickaxeTier::Tier1);
        assert_eq!(
            swing_pickaxe(OreKind::Copper, &mut hits, t1, &mut energy, &mut inventory),
            SwingOutcome::Hit { hits: 1 }
        );
        assert_eq!(
            swing_pickaxe(OreKind::Copper, &mut hits, t1, &mut energy, &mut inventory),
            SwingOutcome::Broke
        );
        assert_eq!(inventory.count(MaterialId::CopperOre), 1);
        assert_eq!(energy.current, energy.max - 8.0);
    }

    #[test]
    fn t1_cannot_touch_iron_and_spends_nothing() {
        let mut energy = full_energy();
        let mut inventory = Inventory::default();
        let mut hits = 0;
        assert_eq!(
            swing_pickaxe(
                OreKind::Iron,
                &mut hits,
                Some(PickaxeTier::Tier1),
                &mut energy,
                &mut inventory
            ),
            SwingOutcome::TooHard
        );
        assert_eq!(hits, 0);
        assert_eq!(energy.current, energy.max);
    }

    #[test]
    fn iron_breaks_in_four_t2_swings() {
        let mut energy = full_energy();
        let mut inventory = Inventory::default();
        let mut hits = 0;
        let t2 = Some(PickaxeTier::Tier2);
        for expected in 1..4 {
            assert_eq!(
                swing_pickaxe(OreKind::Iron, &mut hits, t2, &mut energy, &mut inventory),
                SwingOutcome::Hit { hits: expected }
            );
        }
        assert_eq!(
            swing_pickaxe(OreKind::Iron, &mut hits, t2, &mut energy, &mut inventory),
            SwingOutcome::Broke
        );
        assert_eq!(inventory.count(MaterialId::IronOre), 1);
        assert_eq!(energy.current, energy.max - 12.0);
    }

    #[test]
    fn iron_needs_eight_energy_in_reserve() {
        let mut energy = ToolEnergy::from_saved(7.0, 100.0);
        let mut inventory = Inventory::default();
        let mut hits = 0;
        assert_eq!(
            swing_pickaxe(
                OreKind::Iron,
                &mut hits,
                Some(PickaxeTier::Tier2),
                &mut energy,
                &mut inventory
            ),
            SwingOutcome::NotEnoughEnergy
        );
        assert_eq!(energy.current, 7.0);
        assert_eq!(hits, 0);
    }

    #[test]
    fn best_owned_prefers_t2() {
        let mut inventory = Inventory::default();
        assert_eq!(PickaxeTier::best_owned(&inventory), None);
        inventory.try_add(MaterialId::PickaxeTier1, 1);
        assert_eq!(PickaxeTier::best_owned(&inventory), Some(PickaxeTier::Tier1));
        inventory.try_add(MaterialId::PickaxeTier2, 1);
        assert_eq!(PickaxeTier::best_owned(&inventory), Some(PickaxeTier::Tier2));
    }

    #[test]
    fn depleted_ids_persist_until_sleep() {
        let mut profile = PlayerProfile::default();
        mark_node_depleted(&mut profile, 3);
        mark_node_depleted(&mut profile, 3);
        mark_node_depleted(&mut profile, 9);
        assert_eq!(profile.depleted_ore_nodes, vec![3, 9]);

        let ron = ron::to_string(&profile).expect("serialize");
        let loaded: PlayerProfile = ron::from_str(&ron).expect("deserialize");
        assert_eq!(loaded.depleted_ore_nodes, vec![3, 9]);

        respawn_ore_nodes_on_sleep(&mut profile);
        assert!(profile.depleted_ore_nodes.is_empty());
    }
}
