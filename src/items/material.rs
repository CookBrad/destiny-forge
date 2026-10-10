use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    // Farm goods (#18–20)
    TurnipSeed,
    PotatoSeed,
    Turnip,
    Potato,
    Hoe,
    WateringCan,
    RoastTurnip,
    PotatoStew,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemCategory {
    Tool,
    Seed,
    Crop,
    Food,
    Ore,
    Monster,
}

impl ItemCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Tool => "Tool",
            Self::Seed => "Seed",
            Self::Crop => "Crop",
            Self::Food => "Food",
            Self::Ore => "Ore",
            Self::Monster => "Monster part",
        }
    }
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
            Self::RoastTurnip => "Roast Turnip",
            Self::PotatoStew => "Potato Stew",
        }
    }

    pub fn category(self) -> ItemCategory {
        match self {
            Self::Hoe | Self::WateringCan => ItemCategory::Tool,
            Self::TurnipSeed | Self::PotatoSeed => ItemCategory::Seed,
            Self::Turnip | Self::Potato => ItemCategory::Crop,
            Self::RoastTurnip | Self::PotatoStew => ItemCategory::Food,
            Self::IronScrap => ItemCategory::Ore,
            Self::SlimeGel
            | Self::SlimeCore
            | Self::LeatherWing
            | Self::Fang
            | Self::BoneShard
            | Self::RotFlesh
            | Self::RoyalSlimeCore => ItemCategory::Monster,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::SlimeGel => "Sticky gel carved from slimes. Forge material.",
            Self::SlimeCore => "Dense slime core. Forge material.",
            Self::LeatherWing => "Tough wing leather from bats.",
            Self::Fang => "A sharp fang. Good for spear tips.",
            Self::IronScrap => "Scrap metal from the dungeon. Forge material.",
            Self::BoneShard => "Brittle bone from skeletons.",
            Self::RotFlesh => "Foul flesh from zombies.",
            Self::RoyalSlimeCore => "The King Slime's core. Needed for the Slime Blade.",
            Self::TurnipSeed => "Plant in tilled soil. Ready in 2 days.",
            Self::PotatoSeed => "Plant in tilled soil. Ready in 3 days.",
            Self::Turnip => "Fresh turnip. Cook it at the forge.",
            Self::Potato => "Fresh potato. Cook it at the forge.",
            Self::Hoe => "Tills soil for planting.",
            Self::WateringCan => "Waters crops so they grow overnight.",
            Self::RoastTurnip => "Eat before a hunt. Lasts one hunt or until sleep.",
            Self::PotatoStew => "Eat before a hunt. Lasts one hunt or until sleep.",
        }
    }
}
