use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wallet {
    pub gold: u32,
    pub sparks: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_wallet_starts_empty() {
        let wallet = Wallet::default();
        assert_eq!(wallet.gold, 0);
        assert_eq!(wallet.sparks, 0);
    }
}
