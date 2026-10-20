use std::fmt;

use bevy::prelude::*;
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use super::material::MaterialId;
use super::quality::ItemQuality;

pub const INVENTORY_SLOT_COUNT: usize = 32;
pub const MAX_STACK: u32 = 99;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialStack {
    pub material: Option<MaterialId>,
    #[serde(default)]
    pub quality: ItemQuality,
    pub count: u32,
}

impl MaterialStack {
    fn holds(self, material: MaterialId, quality: ItemQuality) -> bool {
        self.material == Some(material) && self.quality == quality && self.count > 0
    }

    fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Resource, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    #[serde(deserialize_with = "deserialize_slots_padding_shorter_saves")]
    pub slots: [MaterialStack; INVENTORY_SLOT_COUNT],
}

fn deserialize_slots_padding_shorter_saves<'de, D>(
    deserializer: D,
) -> Result<[MaterialStack; INVENTORY_SLOT_COUNT], D::Error>
where
    D: Deserializer<'de>,
{
    struct AnyLengthSlotsVisitor;

    impl<'de> Visitor<'de> for AnyLengthSlotsVisitor {
        type Value = [MaterialStack; INVENTORY_SLOT_COUNT];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a list of inventory slots")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut inventory = Inventory::default();
            let mut overflow_stacks = Vec::new();
            let mut slot_index = 0;
            while let Some(stack) = seq.next_element::<MaterialStack>()? {
                if slot_index < INVENTORY_SLOT_COUNT {
                    inventory.slots[slot_index] = stack;
                } else if let Some(material) = stack.material {
                    overflow_stacks.push((material, stack.quality, stack.count));
                }
                slot_index += 1;
            }
            for (material, quality, count) in overflow_stacks {
                inventory.try_add_quality(material, quality, count);
            }
            Ok(inventory.slots)
        }
    }

    deserializer.deserialize_tuple(INVENTORY_SLOT_COUNT, AnyLengthSlotsVisitor)
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: [MaterialStack::default(); INVENTORY_SLOT_COUNT],
        }
    }
}

impl Inventory {
    pub fn with_starter_kit() -> Self {
        let mut inventory = Self::default();
        inventory.try_add(MaterialId::Hoe, 1);
        inventory.try_add(MaterialId::WateringCan, 1);
        inventory.try_add(MaterialId::PickaxeTier1, 1);
        inventory.try_add(MaterialId::TurnipSeed, 8);
        inventory.try_add(MaterialId::PotatoSeed, 4);
        inventory
    }

    pub fn count(&self, material: MaterialId) -> u32 {
        ItemQuality::LOWEST_FIRST
            .into_iter()
            .map(|quality| self.count_quality(material, quality))
            .sum()
    }

    pub fn count_quality(&self, material: MaterialId, quality: ItemQuality) -> u32 {
        self.slots
            .iter()
            .filter(|slot| slot.holds(material, quality))
            .map(|slot| slot.count)
            .sum()
    }

    pub fn total_items(&self) -> u32 {
        self.slots.iter().map(|slot| slot.count).sum()
    }

    pub fn try_add(&mut self, material: MaterialId, amount: u32) -> u32 {
        self.try_add_quality(material, ItemQuality::Normal, amount)
    }

    pub fn try_add_quality(
        &mut self,
        material: MaterialId,
        quality: ItemQuality,
        amount: u32,
    ) -> u32 {
        if amount == 0 {
            return 0;
        }
        let remaining = self.pour_into_matching(material, quality, amount);
        self.pour_into_empty(material, quality, remaining)
    }

    pub fn try_remove(&mut self, material: MaterialId, amount: u32) -> bool {
        if self.count(material) < amount {
            return false;
        }

        let mut remaining = amount;
        for quality in ItemQuality::LOWEST_FIRST {
            remaining = self.take(material, quality, remaining);
            if remaining == 0 {
                return true;
            }
        }
        false
    }

    pub fn merge_stacks_and_sort_by_category(&mut self) {
        let mut totals = self.totals_by_material_and_quality();
        totals.sort_by_key(|(material, quality, _)| (material.category(), *material, *quality));
        self.slots = [MaterialStack::default(); INVENTORY_SLOT_COUNT];
        for (material, quality, total) in totals {
            self.try_add_quality(material, quality, total);
        }
    }

    pub fn has_room_for(&self, material: MaterialId) -> bool {
        self.has_room_for_quality(material, ItemQuality::Normal)
    }

    pub fn has_room_for_quality(&self, material: MaterialId, quality: ItemQuality) -> bool {
        self.slots.iter().any(|slot| {
            slot.material.is_none() || (slot.holds(material, quality) && slot.count < MAX_STACK)
        })
    }

    fn pour_into_matching(
        &mut self,
        material: MaterialId,
        quality: ItemQuality,
        mut remaining: u32,
    ) -> u32 {
        for slot in &mut self.slots {
            if remaining == 0 {
                return 0;
            }
            if !slot.holds(material, quality) {
                continue;
            }
            let space = MAX_STACK.saturating_sub(slot.count);
            if space == 0 {
                continue;
            }
            let added = remaining.min(space);
            slot.count += added;
            remaining -= added;
        }
        remaining
    }

    fn pour_into_empty(
        &mut self,
        material: MaterialId,
        quality: ItemQuality,
        mut remaining: u32,
    ) -> u32 {
        for slot in &mut self.slots {
            if remaining == 0 {
                return 0;
            }
            if slot.material.is_some() {
                continue;
            }
            let added = remaining.min(MAX_STACK);
            slot.material = Some(material);
            slot.quality = quality;
            slot.count = added;
            remaining -= added;
        }
        remaining
    }

    fn take(&mut self, material: MaterialId, quality: ItemQuality, mut remaining: u32) -> u32 {
        for slot in &mut self.slots {
            if remaining == 0 {
                return 0;
            }
            if !slot.holds(material, quality) {
                continue;
            }
            let removed = remaining.min(slot.count);
            slot.count -= removed;
            remaining -= removed;
            if slot.count == 0 {
                slot.clear();
            }
        }
        remaining
    }

    fn totals_by_material_and_quality(&self) -> Vec<(MaterialId, ItemQuality, u32)> {
        let mut totals = Vec::new();
        for slot in &self.slots {
            let Some(material) = slot.material else {
                continue;
            };
            if slot.count == 0 {
                continue;
            }
            add_total(&mut totals, material, slot.quality, slot.count);
        }
        totals
    }

    pub fn has_materials(&self, costs: &[(MaterialId, u32)]) -> bool {
        costs
            .iter()
            .all(|(material, amount)| self.count(*material) >= *amount)
    }
}

fn add_total(
    totals: &mut Vec<(MaterialId, ItemQuality, u32)>,
    material: MaterialId,
    quality: ItemQuality,
    count: u32,
) {
    match totals
        .iter_mut()
        .find(|(id, stacked_quality, _)| *id == material && *stacked_quality == quality)
    {
        Some((_, _, total)) => *total += count,
        None => totals.push((material, quality, count)),
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
    fn starter_kit_is_hoe_can_pickaxe_turnip_potato_only() {
        let inventory = Inventory::with_starter_kit();
        assert_eq!(inventory.count(MaterialId::Hoe), 1);
        assert_eq!(inventory.count(MaterialId::PickaxeTier1), 1);
        assert_eq!(inventory.count(MaterialId::PickaxeTier2), 0);
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
        let occupied_slots = inventory
            .slots
            .iter()
            .filter(|slot| slot.material.is_some())
            .count();
        assert_eq!(occupied_slots, 4);
    }

    #[test]
    fn sort_merges_partial_stacks_and_groups_by_category() {
        let mut inventory = Inventory::default();
        inventory.slots[0] = MaterialStack {
            material: Some(MaterialId::SlimeGel),
            count: 4,
            ..MaterialStack::default()
        };
        inventory.slots[3] = MaterialStack {
            material: Some(MaterialId::PotatoStew),
            count: 1,
            ..MaterialStack::default()
        };
        inventory.slots[5] = MaterialStack {
            material: Some(MaterialId::SlimeGel),
            count: 6,
            ..MaterialStack::default()
        };
        inventory.slots[9] = MaterialStack {
            material: Some(MaterialId::Hoe),
            count: 1,
            ..MaterialStack::default()
        };
        inventory.slots[12] = MaterialStack {
            material: Some(MaterialId::Turnip),
            count: 2,
            ..MaterialStack::default()
        };
        let items_before_sort = inventory.total_items();

        inventory.merge_stacks_and_sort_by_category();

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
        assert_eq!(inventory.total_items(), items_before_sort);
    }

    #[test]
    fn sort_keeps_overfull_totals_split_at_max_stack() {
        let mut inventory = Inventory::default();
        inventory.try_add(MaterialId::Fang, MAX_STACK + 5);
        inventory.merge_stacks_and_sort_by_category();
        assert_eq!(inventory.slots[0].count, MAX_STACK);
        assert_eq!(inventory.slots[1].count, 5);
    }

    #[test]
    fn short_slot_list_pads_to_slot_count() {
        let old = "(slots: ((material: Some(Turnip), count: 5), (material: None, count: 0)))";
        let inventory: Inventory = ron::from_str(old).unwrap();
        assert_eq!(inventory.slots.len(), INVENTORY_SLOT_COUNT);
        assert_eq!(inventory.count(MaterialId::Turnip), 5);
        assert_eq!(inventory.slots[0].quality, ItemQuality::Normal);
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
        let mut inventory = Inventory::with_starter_kit();
        inventory.try_add(MaterialId::PotatoStew, 2);
        inventory.slots[INVENTORY_SLOT_COUNT - 1] = MaterialStack {
            material: Some(MaterialId::IronScrap),
            quality: ItemQuality::Silver,
            count: 7,
        };
        let text = ron::to_string(&inventory).unwrap();
        let loaded: Inventory = ron::from_str(&text).unwrap();
        assert_eq!(loaded, inventory);
        assert_eq!(
            loaded.slots[INVENTORY_SLOT_COUNT - 1].quality,
            ItemQuality::Silver
        );
    }

    #[test]
    fn different_qualities_stay_in_separate_stacks() {
        let mut inventory = Inventory::default();
        assert_eq!(
            inventory.try_add_quality(MaterialId::Turnip, ItemQuality::Silver, 10),
            0
        );
        assert_eq!(inventory.try_add(MaterialId::Turnip, 4), 0);
        assert_eq!(inventory.count(MaterialId::Turnip), 14);
        assert_eq!(
            inventory.count_quality(MaterialId::Turnip, ItemQuality::Normal),
            4
        );
        assert_eq!(
            inventory.count_quality(MaterialId::Turnip, ItemQuality::Silver),
            10
        );
        assert_eq!(inventory.slots[0].quality, ItemQuality::Silver);
        assert_eq!(inventory.slots[0].count, 10);
        assert_eq!(inventory.slots[1].quality, ItemQuality::Normal);
        assert_eq!(inventory.slots[1].count, 4);
    }

    #[test]
    fn sort_merges_same_quality_and_keeps_other_qualities_apart() {
        let mut inventory = Inventory::default();
        inventory.slots[0] = MaterialStack {
            material: Some(MaterialId::Turnip),
            quality: ItemQuality::Gold,
            count: 2,
        };
        inventory.slots[4] = MaterialStack {
            material: Some(MaterialId::Turnip),
            quality: ItemQuality::Normal,
            count: 3,
        };
        inventory.slots[8] = MaterialStack {
            material: Some(MaterialId::Turnip),
            quality: ItemQuality::Normal,
            count: 4,
        };
        inventory.slots[11] = MaterialStack {
            material: Some(MaterialId::Turnip),
            quality: ItemQuality::Destiny,
            count: 1,
        };

        inventory.merge_stacks_and_sort_by_category();

        assert_eq!(inventory.slots[0].quality, ItemQuality::Normal);
        assert_eq!(inventory.slots[0].count, 7);
        assert_eq!(inventory.slots[1].quality, ItemQuality::Gold);
        assert_eq!(inventory.slots[1].count, 2);
        assert_eq!(inventory.slots[2].quality, ItemQuality::Destiny);
        assert_eq!(inventory.slots[2].count, 1);
        assert!(inventory.slots[3].material.is_none());
        assert_eq!(inventory.count(MaterialId::Turnip), 10);
    }

    #[test]
    fn remove_spends_lower_quality_before_higher_quality() {
        let mut inventory = Inventory::default();
        assert_eq!(
            inventory.try_add_quality(MaterialId::Potato, ItemQuality::Gold, 2),
            0
        );
        assert_eq!(inventory.try_add(MaterialId::Potato, 2), 0);
        assert!(inventory.try_remove(MaterialId::Potato, 3));
        assert_eq!(
            inventory.count_quality(MaterialId::Potato, ItemQuality::Normal),
            0
        );
        assert_eq!(
            inventory.count_quality(MaterialId::Potato, ItemQuality::Gold),
            1
        );
    }

    #[test]
    fn partial_stack_of_another_quality_is_not_room_for_normal() {
        let mut inventory = Inventory::default();
        assert_eq!(
            inventory.try_add_quality(MaterialId::Turnip, ItemQuality::Silver, 1),
            0
        );
        for slot in inventory.slots.iter_mut().skip(1) {
            *slot = MaterialStack {
                material: Some(MaterialId::Fang),
                quality: ItemQuality::Normal,
                count: MAX_STACK,
            };
        }

        assert!(!inventory.has_room_for(MaterialId::Turnip));
        assert!(inventory.has_room_for_quality(MaterialId::Turnip, ItemQuality::Silver));
        assert_eq!(inventory.try_add(MaterialId::Turnip, 1), 1);
        assert_eq!(
            inventory.try_add_quality(MaterialId::Turnip, ItemQuality::Silver, 1),
            0
        );
        assert_eq!(inventory.slots[0].count, 2);
    }

    #[test]
    fn overflow_past_the_slot_list_keeps_its_quality() {
        let mut stacks = vec!["(material: None, count: 0)".to_string(); INVENTORY_SLOT_COUNT];
        stacks.push("(material: Some(Turnip), quality: Gold, count: 6)".to_string());
        let text = format!("(slots: ({}))", stacks.join(", "));
        let inventory: Inventory = ron::from_str(&text).unwrap();
        assert_eq!(
            inventory.count_quality(MaterialId::Turnip, ItemQuality::Gold),
            6
        );
        assert_eq!(
            inventory.count_quality(MaterialId::Turnip, ItemQuality::Normal),
            0
        );
    }
}
