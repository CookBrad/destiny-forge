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
    // Farm goods (#18–20)
    TurnipSeed,
    PotatoSeed,
    Turnip,
    Potato,
    Hoe,
    WateringCan,
    // Mining (#mining-ore-tiers). Raw ore feeds the forge directly — no smelting.
    CopperOre,
    IronOre,
    /// Starter pickaxe: mines copper only.
    PickaxeTier1,
    /// Forged pickaxe: mines copper and iron.
    PickaxeTier2,
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
            Self::CopperOre => "Copper Ore",
            Self::IronOre => "Iron Ore",
            Self::PickaxeTier1 => "Copper Pickaxe",
            Self::PickaxeTier2 => "Iron Pickaxe",
        }
    }
}
