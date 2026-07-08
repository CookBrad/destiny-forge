use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialId {
    SlimeGel,
    SlimeCore,
    LeatherWing,
    Fang,
    IronScrap,
    BoneShard,
    RotFlesh,
    /// Guaranteed King Slime carve; gates Slime Blade.
    RoyalSlimeCore,
    // Farm goods
    TurnipSeed,
    PotatoSeed,
    Turnip,
    Potato,
}

impl MaterialId {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::SlimeGel => "Slime Gel",
            Self::SlimeCore => "Slime Core",
            Self::LeatherWing => "Leather Wing",
            Self::Fang => "Fang",
            Self::IronScrap => "Iron Scrap",
            Self::BoneShard => "Bone Shard",
            Self::RotFlesh => "Rot Flesh",
            Self::RoyalSlimeCore => "Royal Slime Core",
            Self::TurnipSeed => "Turnip Seed",
            Self::PotatoSeed => "Potato Seed",
            Self::Turnip => "Turnip",
            Self::Potato => "Potato",
        }
    }

    /// Longer inventory tooltip text (hover).
    pub fn detail_description(self) -> &'static str {
        match self {
            Self::SlimeGel => "Common slime residue. Used in basic forge recipes.",
            Self::SlimeCore => "Dense slime heart. Needed for slime gear.",
            Self::LeatherWing => "Bat wing leather. Light crafting material.",
            Self::Fang => "Sharp tooth. Used for spear branches.",
            Self::IronScrap => "Rusty metal bits. Iron weapon tier.",
            Self::BoneShard => "Brittle bone fragments from skeletons.",
            Self::RotFlesh => "Unpleasant but useful zombie tissue.",
            Self::RoyalSlimeCore => "King Slime core. Gates the Slime Blade.",
            Self::TurnipSeed => {
                "Plant on tilled soil (Seeds tool or hotbar). Grows in 2 watered days. Harvest with Hand."
            }
            Self::PotatoSeed => {
                "Plant on tilled soil. Grows in 3 watered days. Hearty crop for later cooking."
            }
            Self::Turnip => "Fresh turnip harvest. Cooking uses this later for food buffs.",
            Self::Potato => "Starchy potato harvest. Good cooking ingredient (coming soon).",
        }
    }

    pub fn is_seed(self) -> bool {
        matches!(self, Self::TurnipSeed | Self::PotatoSeed)
    }
}

