use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum ItemQuality {
    #[default]
    Normal,
    Silver,
    Gold,
    Destiny,
}

impl ItemQuality {
    pub(crate) const LOWEST_FIRST: [Self; 4] =
        [Self::Normal, Self::Silver, Self::Gold, Self::Destiny];
}
