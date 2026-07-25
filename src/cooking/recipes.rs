//! Pure cook recipes: crops/fish → food.

use crate::items::{Inventory, MaterialId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CookRecipe {
    pub name: &'static str,
    pub costs: &'static [(MaterialId, u32)],
    pub output: MaterialId,
    pub output_amount: u32,
}

pub const COOK_RECIPES: &[CookRecipe] = &[
    CookRecipe {
        name: "Hearty Stew",
        costs: &[(MaterialId::Turnip, 2), (MaterialId::Potato, 1)],
        output: MaterialId::HeartyStew,
        output_amount: 1,
    },
    CookRecipe {
        name: "Spicy Sashimi",
        costs: &[(MaterialId::RiverFish, 1)],
        output: MaterialId::SpicySashimi,
        output_amount: 1,
    },
];

pub fn can_cook(inventory: &Inventory, recipe: &CookRecipe) -> bool {
    inventory.has_materials(recipe.costs)
}

/// Consume ingredients and grant food. Returns false if materials missing.
pub fn try_cook(inventory: &mut Inventory, recipe: &CookRecipe) -> bool {
    if !can_cook(inventory, recipe) {
        return false;
    }
    for (mat, amount) in recipe.costs {
        if !inventory.try_remove(*mat, *amount) {
            return false;
        }
    }
    let left = inventory.try_add(recipe.output, recipe.output_amount);
    if left > 0 {
        // Rollback on full inventory
        for (mat, amount) in recipe.costs {
            inventory.try_add(*mat, *amount);
        }
        inventory.try_remove(recipe.output, recipe.output_amount.saturating_sub(left));
        return false;
    }
    true
}

pub fn cook_recipe_at(index: usize) -> Option<&'static CookRecipe> {
    COOK_RECIPES.get(index)
}

pub fn cook_recipe_count() -> usize {
    COOK_RECIPES.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stew_requires_crops() {
        let mut inv = Inventory::default();
        let recipe = &COOK_RECIPES[0];
        assert!(!can_cook(&inv, recipe));
        inv.try_add(MaterialId::Turnip, 2);
        inv.try_add(MaterialId::Potato, 1);
        assert!(can_cook(&inv, recipe));
        assert!(try_cook(&mut inv, recipe));
        assert_eq!(inv.count(MaterialId::HeartyStew), 1);
        assert_eq!(inv.count(MaterialId::Turnip), 0);
        assert_eq!(inv.count(MaterialId::Potato), 0);
    }

    #[test]
    fn sashimi_from_fish() {
        let mut inv = Inventory::default();
        let recipe = COOK_RECIPES
            .iter()
            .find(|r| r.output == MaterialId::SpicySashimi)
            .unwrap();
        inv.try_add(MaterialId::RiverFish, 1);
        assert!(try_cook(&mut inv, recipe));
        assert_eq!(inv.count(MaterialId::SpicySashimi), 1);
        assert_eq!(inv.count(MaterialId::RiverFish), 0);
    }

    #[test]
    fn cook_fails_without_materials() {
        let mut inv = Inventory::default();
        assert!(!try_cook(&mut inv, &COOK_RECIPES[0]));
    }
}
