use std::collections::HashMap;

use bevy::prelude::*;

use super::MaterialId;

#[derive(Resource, Default, Clone, Debug)]
pub struct MaterialInventory {
    stacks: HashMap<MaterialId, u32>,
}

impl MaterialInventory {
    pub fn count(&self, material: MaterialId) -> u32 {
        self.stacks.get(&material).copied().unwrap_or(0)
    }

    pub fn add(&mut self, material: MaterialId, amount: u32) {
        if amount == 0 {
            return;
        }
        *self.stacks.entry(material).or_insert(0) += amount;
    }

    pub fn can_remove(&self, requirements: &[(MaterialId, u32)]) -> bool {
        requirements
            .iter()
            .all(|(material, amount)| self.count(*material) >= *amount)
    }

    pub fn remove(&mut self, requirements: &[(MaterialId, u32)]) -> bool {
        if !self.can_remove(requirements) {
            return false;
        }
        for (material, amount) in requirements {
            let entry = self.stacks.get_mut(material).expect("validated above");
            *entry -= amount;
            if *entry == 0 {
                self.stacks.remove(material);
            }
        }
        true
    }

    pub fn iter(&self) -> impl Iterator<Item = (MaterialId, u32)> + '_ {
        self.stacks
            .iter()
            .map(|(material, count)| (*material, *count))
    }
}