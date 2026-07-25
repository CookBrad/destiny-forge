//! Active food buffs: apply on eat, expire on sleep or after one hunt.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::items::MaterialId;

/// When the active buff ends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffExpiry {
    #[default]
    None,
    /// Cleared when the player sleeps.
    UntilSleep,
    /// Cleared when leaving a dungeon hunt (or on sleep).
    OneHunt,
}

/// Runtime + persisted prep buff from cooked food.
#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveFoodBuff {
    pub attack_mult: f32,
    pub defense_bonus: f32,
    pub special_cd_mult: f32,
    pub expiry: BuffExpiry,
    /// Display name of the food that granted this (empty if none).
    #[serde(default)]
    pub source: String,
}

impl Default for ActiveFoodBuff {
    fn default() -> Self {
        Self::none()
    }
}

impl ActiveFoodBuff {
    pub fn none() -> Self {
        Self {
            attack_mult: 1.0,
            defense_bonus: 0.0,
            special_cd_mult: 1.0,
            expiry: BuffExpiry::None,
            source: String::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.expiry != BuffExpiry::None
            && (self.attack_mult != 1.0
                || self.defense_bonus != 0.0
                || self.special_cd_mult != 1.0)
    }

    pub fn attack_multiplier(&self) -> f32 {
        if self.expiry == BuffExpiry::None {
            1.0
        } else {
            self.attack_mult
        }
    }

    pub fn defense_bonus_value(&self) -> f32 {
        if self.expiry == BuffExpiry::None {
            0.0
        } else {
            self.defense_bonus
        }
    }

    pub fn special_cooldown_multiplier(&self) -> f32 {
        if self.expiry == BuffExpiry::None {
            1.0
        } else {
            self.special_cd_mult
        }
    }

    /// Clear buff when sleeping.
    pub fn on_sleep(&mut self) {
        *self = Self::none();
    }

    /// Clear OneHunt buffs when a hunt ends (also safe to call on sleep).
    pub fn on_hunt_end(&mut self) {
        if self.expiry == BuffExpiry::OneHunt {
            *self = Self::none();
        }
    }

    pub fn apply_from_food(&mut self, food: MaterialId) -> bool {
        let Some(effect) = food_effect(food) else {
            return false;
        };
        *self = effect;
        true
    }
}

/// Stat package for a cooked food item.
pub fn food_effect(food: MaterialId) -> Option<ActiveFoodBuff> {
    match food {
        MaterialId::HeartyStew => Some(ActiveFoodBuff {
            attack_mult: 1.0,
            defense_bonus: 4.0,
            special_cd_mult: 1.0,
            expiry: BuffExpiry::UntilSleep,
            source: food.display_name().to_string(),
        }),
        MaterialId::SpicySashimi => Some(ActiveFoodBuff {
            attack_mult: 1.15,
            defense_bonus: 0.0,
            special_cd_mult: 0.95,
            expiry: BuffExpiry::OneHunt,
            source: food.display_name().to_string(),
        }),
        _ => None,
    }
}

/// Eat one unit of food from inventory; applies buff. Returns false if missing/not food.
pub fn try_eat_food(
    inventory: &mut crate::items::Inventory,
    buff: &mut ActiveFoodBuff,
    food: MaterialId,
) -> Result<&'static str, &'static str> {
    if !food.is_food() {
        return Err("not edible");
    }
    if !inventory.try_remove(food, 1) {
        return Err("no food in inventory");
    }
    if !buff.apply_from_food(food) {
        inventory.try_add(food, 1);
        return Err("unknown food");
    }
    Ok(food.display_name())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::Inventory;

    #[test]
    fn stew_gives_defense_until_sleep() {
        let mut inv = Inventory::default();
        inv.try_add(MaterialId::HeartyStew, 1);
        let mut buff = ActiveFoodBuff::none();
        assert!(try_eat_food(&mut inv, &mut buff, MaterialId::HeartyStew).is_ok());
        assert!((buff.defense_bonus_value() - 4.0).abs() < f32::EPSILON);
        assert_eq!(buff.expiry, BuffExpiry::UntilSleep);
        assert_eq!(inv.count(MaterialId::HeartyStew), 0);

        buff.on_hunt_end();
        assert!(buff.is_active()); // until sleep, not one hunt

        buff.on_sleep();
        assert!(!buff.is_active());
        assert!((buff.defense_bonus_value() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn sashimi_attack_expires_after_hunt() {
        let mut inv = Inventory::default();
        inv.try_add(MaterialId::SpicySashimi, 1);
        let mut buff = ActiveFoodBuff::none();
        assert!(try_eat_food(&mut inv, &mut buff, MaterialId::SpicySashimi).is_ok());
        assert!((buff.attack_multiplier() - 1.15).abs() < f32::EPSILON);
        assert_eq!(buff.expiry, BuffExpiry::OneHunt);

        buff.on_hunt_end();
        assert!(!buff.is_active());
        assert!((buff.attack_multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cannot_eat_non_food() {
        let mut inv = Inventory::default();
        inv.try_add(MaterialId::Turnip, 1);
        let mut buff = ActiveFoodBuff::none();
        assert!(try_eat_food(&mut inv, &mut buff, MaterialId::Turnip).is_err());
        assert_eq!(inv.count(MaterialId::Turnip), 1);
    }
}
