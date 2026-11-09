use std::fmt;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DestinySpark(pub u32);

impl DestinySpark {
    pub fn amount(self) -> u32 {
        self.0
    }
}

impl fmt::Display for DestinySpark {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wallet {
    pub gold: u32,
    pub sparks: DestinySpark,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_wallet_starts_empty() {
        let wallet = Wallet::default();
        assert_eq!(wallet.gold, 0);
        assert_eq!(wallet.sparks.amount(), 0);
        assert_eq!(wallet.sparks.to_string(), "0");
    }

    #[test]
    fn wallet_holds_the_largest_amounts() {
        let wallet = Wallet {
            gold: u32::MAX,
            sparks: DestinySpark(u32::MAX),
        };
        assert_eq!(wallet.gold, u32::MAX);
        assert_eq!(wallet.sparks.amount(), u32::MAX);
    }

    #[test]
    fn wallet_round_trips_through_save_text() {
        let wallet = Wallet {
            gold: 250,
            sparks: DestinySpark(4),
        };
        let text = ron::to_string(&wallet).unwrap();
        let loaded: Wallet = ron::from_str(&text).unwrap();
        assert_eq!(loaded, wallet);
        assert!(text.contains("sparks:4"));
    }

    #[test]
    fn wallet_without_sparks_does_not_load() {
        let error = ron::from_str::<Wallet>("(gold:1)").unwrap_err();
        assert!(error.to_string().contains("sparks"));
    }

    #[test]
    fn wallet_rejects_a_spark_amount_that_is_not_a_number() {
        let error = ron::from_str::<Wallet>("(gold:1,sparks:many)").unwrap_err();
        assert!(error.to_string().contains("Expected integer"));
    }
}
