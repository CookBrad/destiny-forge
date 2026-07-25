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
    Pickaxe,
    FishingRod,
    // Mining
    /// Mined metal — primary cost for iron-tier forge recipes.
    IronOre,
    // Fishing
    RiverFish,
    // Cooked food (eat for pre-hunt buffs)
    HeartyStew,
    SpicySashimi,
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
            Self::Pickaxe => "Pickaxe",
            Self::FishingRod => "Fishing Rod",
            Self::IronOre => "Iron Ore",
            Self::RiverFish => "River Fish",
            Self::HeartyStew => "Hearty Stew",
            Self::SpicySashimi => "Spicy Sashimi",
        }
    }

    /// Longer inventory tooltip text (hover).
    pub fn detail_description(self) -> &'static str {
        match self {
            Self::SlimeGel => "Common slime residue. Used in basic forge recipes.",
            Self::SlimeCore => "Dense slime heart. Needed for slime gear.",
            Self::LeatherWing => "Bat wing leather. Light crafting material.",
            Self::Fang => "Sharp tooth. Used for spear branches.",
            Self::IronScrap => "Rusty metal bits from monsters. Secondary metal scrap.",
            Self::BoneShard => "Brittle bone fragments from skeletons.",
            Self::RotFlesh => "Unpleasant but useful zombie tissue.",
            Self::RoyalSlimeCore => "King Slime core. Gates the Slime Blade.",
            Self::TurnipSeed => {
                "Drag to hotbar, select slot, Space on tilled soil. Grows in 2 watered days."
            }
            Self::PotatoSeed => {
                "Drag to hotbar, select slot, Space on tilled soil. Grows in 3 watered days."
            }
            Self::Turnip => "Fresh turnip. Cook into Hearty Stew at the house stove.",
            Self::Potato => "Starchy potato. Cook with turnips into Hearty Stew.",
            Self::Hoe => "Drag to hotbar. Select slot, then Space to till soil for planting.",
            Self::WateringCan => {
                "Drag to hotbar. Select slot, then Space to water planted crops (3 energy)."
            }
            Self::Pickaxe => {
                "Drag to hotbar. Select slot, Space on ore nodes at the mine (8 energy)."
            }
            Self::FishingRod => {
                "Drag to hotbar. Select slot near the dock, Space to cast, Space again to reel."
            }
            Self::IronOre => "Mined ore. Required for iron-tier weapons at the forge.",
            Self::RiverFish => "Fresh catch. Cook into Spicy Sashimi for an attack buff.",
            Self::HeartyStew => "Eat from hotbar (Space). +defense until you sleep.",
            Self::SpicySashimi => "Eat from hotbar (Space). +attack for one hunt.",
        }
    }

    pub fn is_seed(self) -> bool {
        matches!(self, Self::TurnipSeed | Self::PotatoSeed)
    }

    pub fn is_tool(self) -> bool {
        matches!(
            self,
            Self::Hoe | Self::WateringCan | Self::Pickaxe | Self::FishingRod
        )
    }

    pub fn is_food(self) -> bool {
        matches!(self, Self::HeartyStew | Self::SpicySashimi)
    }

    /// Tools are not consumed on use; seeds/crops are stackables.
    pub fn consumed_on_use(self) -> bool {
        self.is_seed() || self.is_food()
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Hoe => "Hoe",
            Self::WateringCan => "Water",
            Self::Pickaxe => "Pick",
            Self::FishingRod => "Rod",
            Self::TurnipSeed => "T.Sd",
            Self::PotatoSeed => "P.Sd",
            Self::Turnip => "Trnp",
            Self::Potato => "Pota",
            Self::IronOre => "Ore",
            Self::RiverFish => "Fish",
            Self::HeartyStew => "Stew",
            Self::SpicySashimi => "Sash",
            Self::SlimeGel => "Gel",
            Self::SlimeCore => "Core",
            Self::LeatherWing => "Wing",
            Self::Fang => "Fang",
            Self::IronScrap => "Scrap",
            Self::BoneShard => "Bone",
            Self::RotFlesh => "Rot",
            Self::RoyalSlimeCore => "Royal",
        }
    }

    pub fn energy_cost(self) -> f32 {
        match self {
            Self::Hoe => 5.0,
            Self::WateringCan => 3.0,
            Self::Pickaxe => 8.0,
            Self::FishingRod => 6.0,
            m if m.is_seed() => 1.0,
            _ => 0.0,
        }
    }

    /// Pickaxe power for ore hardness checks (0 if not a pickaxe).
    pub fn pickaxe_power(self) -> u32 {
        match self {
            Self::Pickaxe => 1,
            _ => 0,
        }
    }
}
