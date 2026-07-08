use crate::combat::WeaponKind;
use crate::items::{Inventory, MaterialId};
use crate::player::{ArmorKind, ArmorSlot, Loadout};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecipeOutput {
    Weapon(WeaponKind),
    Armor(ArmorSlot, ArmorKind),
}

pub struct Recipe {
    pub name: &'static str,
    pub costs: &'static [(MaterialId, u32)],
    pub output: RecipeOutput,
    pub requires_weapon: Option<WeaponKind>,
}

pub const IRON_SWORD_RECIPE: Recipe = Recipe {
    name: "Iron Sword",
    costs: &[(MaterialId::SlimeGel, 5), (MaterialId::IronScrap, 3)],
    output: RecipeOutput::Weapon(WeaponKind::IronSword),
    requires_weapon: None,
};

pub const RUSTY_SPEAR_RECIPE: Recipe = Recipe {
    name: "Rusty Spear",
    costs: &[(MaterialId::SlimeGel, 3), (MaterialId::Fang, 2)],
    output: RecipeOutput::Weapon(WeaponKind::RustySpear),
    requires_weapon: None,
};

pub const SLIME_BLADE_RECIPE: Recipe = Recipe {
    name: "Slime Blade",
    costs: &[(MaterialId::SlimeCore, 2)],
    output: RecipeOutput::Weapon(WeaponKind::SlimeBlade),
    requires_weapon: Some(WeaponKind::IronSword),
};

pub const SLIME_HELM_RECIPE: Recipe = Recipe {
    name: "Slime Helm",
    costs: &[(MaterialId::SlimeGel, 4)],
    output: RecipeOutput::Armor(ArmorSlot::Head, ArmorKind::SlimeHelm),
    requires_weapon: None,
};

pub const SLIME_MAIL_RECIPE: Recipe = Recipe {
    name: "Slime Mail",
    costs: &[(MaterialId::SlimeGel, 6), (MaterialId::SlimeCore, 1)],
    output: RecipeOutput::Armor(ArmorSlot::Chest, ArmorKind::SlimeMail),
    requires_weapon: None,
};

pub const SLIME_GAUNTLETS_RECIPE: Recipe = Recipe {
    name: "Slime Gauntlets",
    costs: &[(MaterialId::SlimeGel, 3)],
    output: RecipeOutput::Armor(ArmorSlot::Arms, ArmorKind::SlimeGauntlets),
    requires_weapon: None,
};

pub const SLIME_GREAVES_RECIPE: Recipe = Recipe {
    name: "Slime Greaves",
    costs: &[(MaterialId::SlimeGel, 3)],
    output: RecipeOutput::Armor(ArmorSlot::Legs, ArmorKind::SlimeGreaves),
    requires_weapon: None,
};

pub const ALL_RECIPES: &[&Recipe] = &[
    &IRON_SWORD_RECIPE,
    &RUSTY_SPEAR_RECIPE,
    &SLIME_BLADE_RECIPE,
    &SLIME_HELM_RECIPE,
    &SLIME_MAIL_RECIPE,
    &SLIME_GAUNTLETS_RECIPE,
    &SLIME_GREAVES_RECIPE,
];

pub fn recipe_set_bonus_hint(recipe: &Recipe) -> Option<&'static str> {
    match recipe.output {
        RecipeOutput::Armor(_, _) => {
            Some(
                "Set: 2pc +10% carve & −10% special CD · 4pc 35% KB resist & +10% attack",
            )
        }
        _ => None,
    }
}

pub fn can_craft_recipe(inventory: &Inventory, loadout: &Loadout, recipe: &Recipe) -> bool {
    if !inventory.has_materials(recipe.costs) {
        return false;
    }

    if let Some(required_weapon) = recipe.requires_weapon {
        if loadout.weapon != required_weapon {
            return false;
        }
    }

    true
}

pub fn try_craft_recipe(inventory: &mut Inventory, loadout: &mut Loadout, recipe: &Recipe) -> bool {
    if !can_craft_recipe(inventory, loadout, recipe) {
        return false;
    }

    for (material, amount) in recipe.costs {
        if !inventory.try_remove(*material, *amount) {
            return false;
        }
    }

    match recipe.output {
        RecipeOutput::Weapon(weapon) => loadout.weapon = weapon,
        RecipeOutput::Armor(_, armor) => loadout.armor.set(armor),
    }

    true
}

pub fn try_craft_iron_sword(inventory: &mut Inventory, loadout: &mut Loadout) -> bool {
    try_craft_recipe(inventory, loadout, &IRON_SWORD_RECIPE)
}

pub fn material_name(material: MaterialId) -> &'static str {
    match material {
        MaterialId::SlimeGel => "Slime Gel",
        MaterialId::SlimeCore => "Slime Core",
        MaterialId::LeatherWing => "Leather Wing",
        MaterialId::Fang => "Fang",
        MaterialId::IronScrap => "Iron Scrap",
    }
}

pub fn recipe_costs_text(inventory: &Inventory, recipe: &Recipe) -> String {
    recipe
        .costs
        .iter()
        .map(|(material, amount)| {
            format!(
                "{} {}/{}",
                material_name(*material),
                inventory.count(*material),
                amount
            )
        })
        .collect::<Vec<_>>()
        .join("  ·  ")
}

pub fn recipe_requirement_text(loadout: &Loadout, recipe: &Recipe) -> Option<String> {
    recipe.requires_weapon.map(|weapon| {
        let label = match weapon {
            WeaponKind::RustySword => "Rusty Sword",
            WeaponKind::RustySpear => "Rusty Spear",
            WeaponKind::IronSword => "Iron Sword",
            WeaponKind::SlimeBlade => "Slime Blade",
        };
        if loadout.weapon == weapon {
            format!("Requires equipped {label} (ready)")
        } else {
            format!("Requires equipped {label}")
        }
    })
}

pub fn forge_status(inventory: &Inventory, loadout: &Loadout, recipe: &Recipe) -> String {
    if can_craft_recipe(inventory, loadout, recipe) {
        format!("{} is ready to craft.", recipe.name)
    } else if let Some(requirement) = recipe_requirement_text(loadout, recipe) {
        if loadout.weapon != recipe.requires_weapon.unwrap_or(loadout.weapon) {
            requirement
        } else {
            "Gather the required materials.".to_string()
        }
    } else {
        "Gather the required materials.".to_string()
    }
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

    #[test]
    fn slime_blade_requires_iron_sword_equipped() {
        let mut inventory = Inventory::default();
        let mut loadout = Loadout::default();
        inventory.try_add(MaterialId::SlimeCore, 2);

        assert!(!try_craft_recipe(&mut inventory, &mut loadout, &SLIME_BLADE_RECIPE));

        loadout.weapon = WeaponKind::IronSword;
        assert!(try_craft_recipe(&mut inventory, &mut loadout, &SLIME_BLADE_RECIPE));
        assert_eq!(loadout.weapon, WeaponKind::SlimeBlade);
    }

    #[test]
    fn armor_piece_equips_into_slot() {
        let mut inventory = Inventory::default();
        let mut loadout = Loadout::default();
        inventory.try_add(MaterialId::SlimeGel, 4);

        assert!(try_craft_recipe(&mut inventory, &mut loadout, &SLIME_HELM_RECIPE));
        assert_eq!(loadout.armor.head, Some(ArmorKind::SlimeHelm));
    }
}