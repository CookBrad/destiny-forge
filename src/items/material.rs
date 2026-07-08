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
    // Homestead tools (inventory + hotbar)
    Hoe,
    WateringCan,
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
            Self::Hoe => "Hoe",
            Self::WateringCan => "Watering Can",
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
                "Drag to hotbar, select slot, Space on tilled soil. Grows in 2 watered days."
            }
            Self::PotatoSeed => {
                "Drag to hotbar, select slot, Space on tilled soil. Grows in 3 watered days."
            }
            Self::Turnip => "Fresh turnip harvest. Cooking uses this later for food buffs.",
            Self::Potato => "Starchy potato harvest. Good cooking ingredient (coming soon).",
            Self::Hoe => "Drag to hotbar. Select slot, then Space to till soil for planting.",
            Self::WateringCan => {
                "Drag to hotbar. Select slot, then Space to water planted crops (3 energy)."
            }
        }
    }

    pub fn is_seed(self) -> bool {
        matches!(self, Self::TurnipSeed | Self::PotatoSeed)
    }

    pub fn is_tool(self) -> bool {
        matches!(self, Self::Hoe | Self::WateringCan)
    }

    /// Tools are not consumed on use; seeds/crops are stackables.
    pub fn consumed_on_use(self) -> bool {
        self.is_seed()
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Hoe => "Hoe",
            Self::WateringCan => "Water",
            Self::TurnipSeed => "T.Sd",
            Self::PotatoSeed => "P.Sd",
            Self::Turnip => "Trnp",
            Self::Potato => "Pota",
            other => {
                // Fall back to first word of display name for hotbar.
                match other {
                    Self::SlimeGel => "Gel",
                    Self::SlimeCore => "Core",
                    Self::LeatherWing => "Wing",
                    Self::Fang => "Fang",
                    Self::IronScrap => "Iron",
                    Self::BoneShard => "Bone",
                    Self::RotFlesh => "Rot",
                    Self::RoyalSlimeCore => "Royal",
                    _ => "?",
                }
            }
        }
    }

    pub fn energy_cost(self) -> f32 {
        match self {
            Self::Hoe => 5.0,
            Self::WateringCan => 3.0,
            m if m.is_seed() => 1.0,
            _ => 0.0,
        }
    }
}

