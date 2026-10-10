use super::material::MaterialId;

/// Pre-hunt prep bonus from eating one food. Small on purpose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoodBuff {
    pub attack_mult: f32,
    pub defense_bonus: f32,
    pub carve_mult: f32,
}

impl FoodBuff {
    pub const NONE: Self = Self {
        attack_mult: 1.0,
        defense_bonus: 0.0,
        carve_mult: 1.0,
    };

    pub fn summary(self) -> String {
        let mut parts = Vec::new();
        if self.attack_mult != 1.0 {
            parts.push(format!("+{:.0}% attack", (self.attack_mult - 1.0) * 100.0));
        }
        if self.defense_bonus != 0.0 {
            parts.push(format!("+{:.0} defense", self.defense_bonus));
        }
        if self.carve_mult != 1.0 {
            parts.push(format!("+{:.0}% carve", (self.carve_mult - 1.0) * 100.0));
        }
        parts.join(", ")
    }
}

impl MaterialId {
    pub fn food_buff(self) -> Option<FoodBuff> {
        match self {
            Self::RoastTurnip => Some(FoodBuff {
                attack_mult: 1.08,
                ..FoodBuff::NONE
            }),
            Self::PotatoStew => Some(FoodBuff {
                defense_bonus: 2.0,
                carve_mult: 1.1,
                ..FoodBuff::NONE
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::ItemCategory;

    #[test]
    fn only_food_items_have_buffs() {
        assert!(MaterialId::Turnip.food_buff().is_none());
        assert!(MaterialId::SlimeGel.food_buff().is_none());
        for food in [MaterialId::RoastTurnip, MaterialId::PotatoStew] {
            assert_eq!(food.category(), ItemCategory::Food);
            assert!(food.food_buff().is_some());
        }
    }

    #[test]
    fn buff_summaries_list_each_bonus() {
        let roast = MaterialId::RoastTurnip.food_buff().unwrap();
        assert_eq!(roast.summary(), "+8% attack");
        let stew = MaterialId::PotatoStew.food_buff().unwrap();
        assert_eq!(stew.summary(), "+2 defense, +10% carve");
    }
}
