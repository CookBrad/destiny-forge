use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ArmorSlot {
    Head,
    Chest,
    Arms,
    Legs,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ArmorId {
    SlimeHelm,
    SlimeMail,
    SlimeGauntlets,
    SlimeGreaves,
}

impl ArmorId {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::SlimeHelm => "Slime Helm",
            Self::SlimeMail => "Slime Mail",
            Self::SlimeGauntlets => "Slime Gauntlets",
            Self::SlimeGreaves => "Slime Greaves",
        }
    }

    pub fn slot(self) -> ArmorSlot {
        match self {
            Self::SlimeHelm => ArmorSlot::Head,
            Self::SlimeMail => ArmorSlot::Chest,
            Self::SlimeGauntlets => ArmorSlot::Arms,
            Self::SlimeGreaves => ArmorSlot::Legs,
        }
    }

    pub fn defense(self) -> f32 {
        match self {
            Self::SlimeHelm => 2.0,
            Self::SlimeMail => 5.0,
            Self::SlimeGauntlets => 2.0,
            Self::SlimeGreaves => 3.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ArmorPiece {
    pub id: ArmorId,
}