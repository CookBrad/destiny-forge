use std::fmt;

use bevy::prelude::*;
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use super::material::MaterialId;

/// Raised from 24 in profile v7. Saved slot lists of any length still load.
pub const INVENTORY_SLOT_COUNT: usize = 32;
pub const MAX_STACK: u32 = 99;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialStack {
    pub material: Option<MaterialId>,
    pub count: u32,
}

impl Default for MaterialStack {
    fn default() -> Self {
        Self {
            material: None,
            count: 0,
        }
    }
}

#[derive(Resource, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    #[serde(deserialize_with = "deserialize_slots")]
    pub slots: [MaterialStack; INVENTORY_SLOT_COUNT],
}

/// Pads shorter (pre-v7, 24-slot) saves with empty slots. Stacks beyond the
/// slot count are merged back in rather than dropped.
fn deserialize_slots<'de, D>(
    deserializer: D,
) -> Result<[MaterialStack; INVENTORY_SLOT_COUNT], D::Error>
where
    D: Deserializer<'de>,
{
    struct SlotsVisitor;

    impl<'de> Visitor<'de> for SlotsVisitor {
        type Value = [MaterialStack; INVENTORY_SLOT_COUNT];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a list of inventory slots")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut inventory = Inventory::default();
            let mut overflow = Vec::new();
            let mut index = 0;
            while let Some(stack) = seq.next_element::<MaterialStack>()? {
                if index < INVENTORY_SLOT_COUNT {
                    inventory.slots[index] = stack;
                } else if let Some(material) = stack.material {
                    overflow.push((material, stack.count));
                }
                index += 1;
            }
            for (material, count) in overflow {
                inventory.try_add(material, count);
            }
            Ok(inventory.slots)
        }
    }

    deserializer.deserialize_tuple(INVENTORY_SLOT_COUNT, SlotsVisitor)
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: [MaterialStack::default(); INVENTORY_SLOT_COUNT],
        }
    }
}

impl Inventory {
    /// Hoe, watering can, and starter seeds for a new homestead profile.
    /// Does not grant pickaxe or fishing rod.
    pub fn with_starter_seeds() -> Self {
        let mut inventory = Self::default();
        inventory.try_add(MaterialId::Hoe, 1);
        inventory.try_add(MaterialId::WateringCan, 1);
        inventory.try_add(MaterialId::TurnipSeed, 8);
        inventory.try_add(MaterialId::PotatoSeed, 4);
        inventory
    }

    pub fn count(&self, material: MaterialId) -> u32 {
        self.slots
            .iter()
            .filter(|slot| slot.material == Some(material))
            .map(|slot| slot.count)
            .sum()
    }

    pub fn total_items(&self) -> u32 {
        self.slots.iter().map(|slot| slot.count).sum()
    }

    pub fn try_add(&mut self, material: MaterialId, amount: u32) -> u32 {
        if amount == 0 {
            return 0;
        }

        let mut remaining = amount;

        for slot in &mut self.slots {
            if slot.material != Some(material) {
                continue;
            }
            let space = MAX_STACK.saturating_sub(slot.count);
            if space == 0 {
                continue;
            }
            let added = remaining.min(space);
            slot.count += added;
            remaining -= added;
            if remaining == 0 {
                return 0;
            }
        }

        for slot in &mut self.slots {
            if slot.material.is_some() {
                continue;
            }
            let added = remaining.min(MAX_STACK);
            slot.material = Some(material);
            slot.count = added;
            remaining -= added;
            if remaining == 0 {
                return 0;
            }
        }

        remaining
    }

    pub fn try_remove(&mut self, material: MaterialId, amount: u32) -> bool {
        if self.count(material) < amount {
            return false;
        }

        let mut remaining = amount;
        for slot in &mut self.slots {
            if slot.material != Some(material) || slot.count == 0 {
                continue;
            }
            let removed = remaining.min(slot.count);
            slot.count -= removed;
            remaining -= removed;
            if slot.count == 0 {
                slot.material = None;
            }
            if remaining == 0 {
                return true;
            }
        }

        false
    }

    /// Merges partial stacks and groups by category, then item.
    pub fn sort(&mut self) {
        let mut totals: Vec<(MaterialId, u32)> = Vec::new();
        for slot in &self.slots {
            let Some(material) = slot.material else {
                continue;
            };
            if slot.count == 0 {
                continue;
            }
            match totals.iter_mut().find(|(id, _)| *id == material) {
                Some((_, total)) => *total += slot.count,
                None => totals.push((material, slot.count)),
            }
        }
        totals.sort_by_key(|(material, _)| (material.category(), *material));

        self.slots = [MaterialStack::default(); INVENTORY_SLOT_COUNT];
        for (material, total) in totals {
            self.try_add(material, total);
        }
    }

    pub fn has_materials(&self, costs: &[(MaterialId, u32)]) -> bool {
        costs
            .iter()
            .all(|(material, amount)| self.count(*material) >= *amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_remove_materials() {
        let mut inventory = Inventory::default();
        assert_eq!(inventory.try_add(MaterialId::SlimeGel, 5), 0);
        assert_eq!(inventory.count(MaterialId::SlimeGel), 5);
        assert!(inventory.try_remove(MaterialId::SlimeGel, 3));
        assert_eq!(inventory.count(MaterialId::SlimeGel), 2);
        assert!(!inventory.try_remove(MaterialId::SlimeGel, 3));
    }

    #[test]
    fn stacks_same_material() {
        let mut inventory = Inventory::default();
        assert_eq!(inventory.try_add(MaterialId::Fang, 99), 0);
        assert_eq!(inventory.count(MaterialId::Fang), 99);
        assert_eq!(inventory.try_add(MaterialId::Fang, 1), 0);
        assert_eq!(inventory.count(MaterialId::Fang), 100);
    }

    #[test]
    fn starter_seeds_are_hoe_can_turnip_potato_only() {
        let inventory = Inventory::with_starter_seeds();
        assert_eq!(inventory.count(MaterialId::Hoe), 1);
        assert_eq!(inventory.count(MaterialId::WateringCan), 1);
        assert_eq!(inventory.count(MaterialId::TurnipSeed), 8);
        assert_eq!(inventory.count(MaterialId::PotatoSeed), 4);
        assert_eq!(inventory.count(MaterialId::Turnip), 0);
        assert_eq!(inventory.count(MaterialId::Potato), 0);
    }

    #[test]
    fn crop_food_and_ore_ids_stack() {
        let mut inventory = Inventory::default();
        for material in [
            MaterialId::Turnip,
            MaterialId::RoastTurnip,
            MaterialId::PotatoStew,
            MaterialId::IronScrap,
        ] {
            assert_eq!(inventory.try_add(material, 3), 0);
            assert_eq!(inventory.try_add(material, 4), 0);
            assert_eq!(inventory.count(material), 7);
        }
        let used = inventory
            .slots
            .iter()
            .filter(|slot| slot.material.is_some())
            .count();
        assert_eq!(used, 4);
    }

    #[test]
    fn sort_merges_partial_stacks_and_groups_by_category() {
        let mut inventory = Inventory::default();
        inventory.slots[0] = MaterialStack {
            material: Some(MaterialId::SlimeGel),
            count: 4,
        };
        inventory.slots[3] = MaterialStack {
            material: Some(MaterialId::PotatoStew),
            count: 1,
        };
        inventory.slots[5] = MaterialStack {
            material: Some(MaterialId::SlimeGel),
            count: 6,
        };
        inventory.slots[9] = MaterialStack {
            material: Some(MaterialId::Hoe),
            count: 1,
        };
        inventory.slots[12] = MaterialStack {
            material: Some(MaterialId::Turnip),
            count: 2,
        };
        let before = inventory.total_items();

        inventory.sort();

        let order: Vec<_> = inventory.slots.iter().map(|slot| slot.material).collect();
        assert_eq!(
            &order[..5],
            &[
                Some(MaterialId::Hoe),
                Some(MaterialId::Turnip),
                Some(MaterialId::PotatoStew),
                Some(MaterialId::SlimeGel),
                None,
            ]
        );
        assert_eq!(inventory.slots[3].count, 10);
        assert_eq!(inventory.total_items(), before);
    }

    #[test]
    fn sort_keeps_overfull_totals_split_at_max_stack() {
        let mut inventory = Inventory::default();
        inventory.try_add(MaterialId::Fang, MAX_STACK + 5);
        inventory.sort();
        assert_eq!(inventory.slots[0].count, MAX_STACK);
        assert_eq!(inventory.slots[1].count, 5);
    }

    #[test]
    fn short_slot_list_pads_to_slot_count() {
        let old = "(slots: ((material: Some(Turnip), count: 5), (material: None, count: 0)))";
        let inventory: Inventory = ron::from_str(old).unwrap();
        assert_eq!(inventory.slots.len(), INVENTORY_SLOT_COUNT);
        assert_eq!(inventory.count(MaterialId::Turnip), 5);
    }

    #[test]
    fn long_slot_list_merges_overflow_instead_of_dropping() {
        let mut stacks = vec!["(material: None, count: 0)".to_string(); INVENTORY_SLOT_COUNT];
        stacks[0] = "(material: Some(Fang), count: 2)".to_string();
        stacks.push("(material: Some(Fang), count: 3)".to_string());
        let text = format!("(slots: ({}))", stacks.join(", "));
        let inventory: Inventory = ron::from_str(&text).unwrap();
        assert_eq!(inventory.count(MaterialId::Fang), 5);
    }

    #[test]
    fn round_trips_through_ron() {
        let mut inventory = Inventory::with_starter_seeds();
        inventory.try_add(MaterialId::PotatoStew, 2);
        inventory.slots[INVENTORY_SLOT_COUNT - 1] = MaterialStack {
            material: Some(MaterialId::IronScrap),
            count: 7,
        };
        let text = ron::to_string(&inventory).unwrap();
        let loaded: Inventory = ron::from_str(&text).unwrap();
        assert_eq!(loaded, inventory);
    }
}
