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
        }
    }
}
