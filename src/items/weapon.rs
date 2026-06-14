use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum WeaponId {
    RustySword,
    IronSword,
    SlimeBlade,
    RustySpear,
}

impl WeaponId {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::RustySword => "Rusty Sword",
            Self::IronSword => "Iron Sword",
            Self::SlimeBlade => "Slime Blade",
            Self::RustySpear => "Rusty Spear",
        }
    }

    pub fn damage(self) -> f32 {
        match self {
            Self::RustySword => 10.0,
            Self::IronSword => 18.0,
            Self::SlimeBlade => 28.0,
            Self::RustySpear => 14.0,
        }
    }

    pub fn reach(self) -> f32 {
        match self {
            Self::RustySword => 36.0,
            Self::IronSword => 40.0,
            Self::SlimeBlade => 44.0,
            Self::RustySpear => 56.0,
        }
    }
}