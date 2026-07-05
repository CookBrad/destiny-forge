use crate::combat::WeaponKind;
use crate::items::{Inventory, MaterialId};
use crate::player::Loadout;

pub struct Recipe {
    pub name: &'static str,
    pub costs: &'static [(MaterialId, u32)],
    pub output_weapon: Option<WeaponKind>,
}

pub const IRON_SWORD_RECIPE: Recipe = Recipe {
    name: "Iron Sword",
    costs: &[(MaterialId::SlimeGel, 5), (MaterialId::IronScrap, 3)],
    output_weapon: Some(WeaponKind::IronSword),
};

pub fn try_craft_iron_sword(inventory: &mut Inventory, loadout: &mut Loadout) -> bool {
    try_craft_recipe(inventory, loadout, &IRON_SWORD_RECIPE)
}

pub fn try_craft_recipe(inventory: &mut Inventory, loadout: &mut Loadout, recipe: &Recipe) -> bool {
    if !inventory.has_materials(recipe.costs) {
        return false;
    }

    for (material, amount) in recipe.costs {
        if !inventory.try_remove(*material, *amount) {
            return false;
        }
    }

    if let Some(weapon) = recipe.output_weapon {
        loadout.weapon = weapon;
    }

    true
}

pub fn can_craft_recipe(inventory: &Inventory, recipe: &Recipe) -> bool {
    inventory.has_materials(recipe.costs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crafts_iron_sword_when_materials_available() {
        let mut inventory = Inventory::default();
        let mut loadout = Loadout::default();
        inventory.try_add(MaterialId::SlimeGel, 5);
        inventory.try_add(MaterialId::IronScrap, 3);

        assert!(try_craft_iron_sword(&mut inventory, &mut loadout));
        assert_eq!(loadout.weapon, WeaponKind::IronSword);
        assert_eq!(inventory.count(MaterialId::SlimeGel), 0);
    }
}