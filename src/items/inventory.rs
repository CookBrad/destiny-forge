use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::material::MaterialId;

pub const INVENTORY_SLOT_COUNT: usize = 24;
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
    pub slots: [MaterialStack; INVENTORY_SLOT_COUNT],
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: [MaterialStack::default(); INVENTORY_SLOT_COUNT],
        }
    }
}

impl Inventory {
    /// Starter seeds for a new homestead profile.
    pub fn with_starter_seeds() -> Self {
        let mut inventory = Self::default();
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
}