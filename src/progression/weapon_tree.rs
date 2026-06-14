use crate::items::WeaponId;

#[derive(Clone, Copy, Debug)]
pub struct WeaponUpgrade {
    pub from: Option<WeaponId>,
    pub to: WeaponId,
}

pub struct WeaponUpgradeTree;

impl WeaponUpgradeTree {
    pub const UPGRADES: [WeaponUpgrade; 3] = [
        WeaponUpgrade {
            from: Some(WeaponId::RustySword),
            to: WeaponId::IronSword,
        },
        WeaponUpgrade {
            from: Some(WeaponId::IronSword),
            to: WeaponId::SlimeBlade,
        },
        WeaponUpgrade {
            from: None,
            to: WeaponId::RustySpear,
        },
    ];

    pub fn consumes_weapon(recipe_output: WeaponId) -> Option<WeaponId> {
        Self::UPGRADES
            .iter()
            .find(|upgrade| upgrade.to == recipe_output)
            .and_then(|upgrade| upgrade.from)
    }
}