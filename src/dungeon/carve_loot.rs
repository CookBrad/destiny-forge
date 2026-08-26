//! Pure carve loot resolution — unit-testable without Bevy systems.
//! Tables load from `assets/data/carve_loot.ron`.

use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::core::data_load::load_ron_from_assets_or_embedded;
use crate::items::MaterialId;

use super::enemy::EnemyKind;

const LOOT_PATH: &str = "assets/data/carve_loot.ron";
const EMBEDDED_LOOT: &str = include_str!("../../assets/data/carve_loot.ron");

/// What is being carved: trash pack or boss.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CarveTarget {
    Pack(EnemyKind),
    KingSlime,
}

impl CarveTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pack(EnemyKind::Slime) => "slime",
            Self::Pack(EnemyKind::Bat) => "bat",
            Self::Pack(EnemyKind::Goblin) => "goblin",
            Self::Pack(EnemyKind::Skeleton) => "skeleton",
            Self::Pack(EnemyKind::Zombie) => "zombie",
            Self::KingSlime => "king slime",
        }
    }
}

/// Named rarity for bonus carve parts. Set skills can later scale Rare weight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LootRarity {
    Common,
    Uncommon,
    Rare,
}

/// Integer weights for Common / Uncommon / Rare bonus rolls.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RarityWeights {
    pub common: u32,
    pub uncommon: u32,
    pub rare: u32,
}

impl Default for RarityWeights {
    fn default() -> Self {
        Self {
            common: 70,
            uncommon: 25,
            rare: 5,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BonusDrop {
    pub material: MaterialId,
    pub amount: u32,
    pub rarity: LootRarity,
}

/// Guaranteed drops always granted; bonus rolls pick a rarity then a part.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LootTable {
    pub target: CarveTarget,
    pub guaranteed: Vec<(MaterialId, u32)>,
    pub bonus: Vec<BonusDrop>,
    #[serde(default)]
    pub rarity_weights: RarityWeights,
    #[serde(default = "default_bonus_rolls")]
    pub bonus_rolls: u32,
}

fn default_bonus_rolls() -> u32 {
    1
}

/// Runtime loot tables loaded from RON.
#[derive(Resource, Clone, Debug)]
pub struct CarveLootBook {
    tables: Vec<LootTable>,
}

impl Default for CarveLootBook {
    fn default() -> Self {
        Self::load()
    }
}

impl CarveLootBook {
    pub fn load() -> Self {
        match load_ron_from_assets_or_embedded::<Vec<LootTable>>(
            LOOT_PATH,
            EMBEDDED_LOOT,
            "carve loot",
        ) {
            Some(tables) if !tables.is_empty() => Self { tables },
            Some(_) => {
                bevy::log::error!("Carve loot tables empty");
                Self {
                    tables: Vec::new(),
                }
            }
            None => Self {
                tables: Vec::new(),
            },
        }
    }

    pub fn table_for(&self, target: CarveTarget) -> Option<&LootTable> {
        self.tables.iter().find(|table| table.target == target)
    }

    /// Roll carve yields for a target. Always includes guaranteed parts when a table exists.
    pub fn roll(&self, target: CarveTarget, rng: &mut impl Rng) -> Vec<(MaterialId, u32)> {
        self.roll_with_rare_bonus(target, rng, 1.0)
    }

    /// Like [`Self::roll`], with Rare weight scaled by `rare_chance_multiplier` (set-skill hook).
    pub fn roll_with_rare_bonus(
        &self,
        target: CarveTarget,
        rng: &mut impl Rng,
        rare_chance_multiplier: f32,
    ) -> Vec<(MaterialId, u32)> {
        let Some(table) = self.table_for(target) else {
            bevy::log::warn!("No carve loot table for {target:?}");
            return Vec::new();
        };
        roll_table(table, rng, rare_chance_multiplier)
    }

    /// Deterministic loot for tests / debugging (every bonus part granted).
    pub fn max_loot(&self, target: CarveTarget) -> Vec<(MaterialId, u32)> {
        let Some(table) = self.table_for(target) else {
            return Vec::new();
        };
        let mut drops = Vec::new();
        for &(material, amount) in &table.guaranteed {
            push_or_stack(&mut drops, material, amount);
        }
        for drop in &table.bonus {
            push_or_stack(&mut drops, drop.material, drop.amount);
        }
        drops
    }
}

/// Convenience for call sites that hold a book resource.
pub fn roll_carve_loot(
    book: &CarveLootBook,
    target: CarveTarget,
    rng: &mut impl Rng,
) -> Vec<(MaterialId, u32)> {
    book.roll(target, rng)
}

fn roll_table(
    table: &LootTable,
    rng: &mut impl Rng,
    rare_chance_multiplier: f32,
) -> Vec<(MaterialId, u32)> {
    let mut drops = Vec::with_capacity(table.guaranteed.len() + table.bonus_rolls as usize);

    for &(material, amount) in &table.guaranteed {
        push_or_stack(&mut drops, material, amount);
    }

    for _ in 0..table.bonus_rolls {
        let Some(rarity) = pick_rarity(table.rarity_weights, rare_chance_multiplier, rng) else {
            continue;
        };
        let Some(drop) = pick_bonus(table, rarity, rng) else {
            continue;
        };
        push_or_stack(&mut drops, drop.material, drop.amount);
    }

    drops
}

fn pick_rarity(
    weights: RarityWeights,
    rare_chance_multiplier: f32,
    rng: &mut impl Rng,
) -> Option<LootRarity> {
    rarity_at(weights, rare_chance_multiplier, rng.gen::<f32>())
}

/// Map a unit roll in `0.0..=1.0` onto weighted Common / Uncommon / Rare.
fn rarity_at(
    weights: RarityWeights,
    rare_chance_multiplier: f32,
    unit_roll: f32,
) -> Option<LootRarity> {
    let common = weights.common as f64;
    let uncommon = weights.uncommon as f64;
    let rare = (weights.rare as f64) * (rare_chance_multiplier.max(0.0) as f64);
    let total = common + uncommon + rare;
    if total <= 0.0 {
        return None;
    }

    let bands = [
        (LootRarity::Common, common),
        (LootRarity::Uncommon, uncommon),
        (LootRarity::Rare, rare),
    ];
    let target = (unit_roll.clamp(0.0, 1.0) as f64) * total;
    let mut acc = 0.0;
    for (rarity, weight) in bands {
        acc += weight;
        if target < acc {
            return Some(rarity);
        }
    }
    bands
        .iter()
        .rev()
        .find(|(_, weight)| *weight > 0.0)
        .map(|(rarity, _)| *rarity)
}

fn pick_bonus<'a>(table: &'a LootTable, rarity: LootRarity, rng: &mut impl Rng) -> Option<&'a BonusDrop> {
    let pool: Vec<&BonusDrop> = table
        .bonus
        .iter()
        .filter(|drop| drop.rarity == rarity)
        .collect();
    if pool.is_empty() {
        return None;
    }
    let index = rng.gen_range(0..pool.len());
    Some(pool[index])
}

fn push_or_stack(drops: &mut Vec<(MaterialId, u32)>, material: MaterialId, amount: u32) {
    if let Some((_, total)) = drops.iter_mut().find(|(id, _)| *id == material) {
        *total += amount;
    } else {
        drops.push((material, amount));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn book() -> CarveLootBook {
        CarveLootBook::load()
    }

    fn bonus_rarities(table: &LootTable) -> Vec<LootRarity> {
        let mut rarities: Vec<_> = table.bonus.iter().map(|drop| drop.rarity).collect();
        rarities.sort();
        rarities.dedup();
        rarities
    }

    #[test]
    fn each_pack_species_has_distinct_guaranteed_part() {
        let book = book();
        let slime = book.table_for(CarveTarget::Pack(EnemyKind::Slime)).unwrap();
        let bat = book.table_for(CarveTarget::Pack(EnemyKind::Bat)).unwrap();
        let goblin = book.table_for(CarveTarget::Pack(EnemyKind::Goblin)).unwrap();
        let skeleton = book
            .table_for(CarveTarget::Pack(EnemyKind::Skeleton))
            .unwrap();
        let zombie = book.table_for(CarveTarget::Pack(EnemyKind::Zombie)).unwrap();

        assert!(slime
            .guaranteed
            .iter()
            .any(|(m, _)| *m == MaterialId::SlimeGel));
        assert!(bat
            .guaranteed
            .iter()
            .any(|(m, _)| *m == MaterialId::LeatherWing));
        assert!(goblin
            .guaranteed
            .iter()
            .any(|(m, _)| *m == MaterialId::Fang));
        assert!(skeleton
            .guaranteed
            .iter()
            .any(|(m, _)| *m == MaterialId::BoneShard));
        assert!(zombie
            .guaranteed
            .iter()
            .any(|(m, _)| *m == MaterialId::RotFlesh));
    }

    #[test]
    fn king_slime_always_drops_royal_core() {
        let book = book();
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..20 {
            let drops = book.roll(CarveTarget::KingSlime, &mut rng);
            let royal: u32 = drops
                .iter()
                .filter(|(m, _)| *m == MaterialId::RoyalSlimeCore)
                .map(|(_, n)| *n)
                .sum();
            assert!(
                royal >= 1,
                "boss must always yield at least one Royal Slime Core"
            );
        }
    }

    #[test]
    fn roll_never_empty_for_packs() {
        let book = book();
        let mut rng = StdRng::seed_from_u64(42);
        for kind in [
            EnemyKind::Slime,
            EnemyKind::Bat,
            EnemyKind::Goblin,
            EnemyKind::Skeleton,
            EnemyKind::Zombie,
        ] {
            let drops = book.roll(CarveTarget::Pack(kind), &mut rng);
            assert!(!drops.is_empty(), "{kind:?} should always yield something");
        }
    }

    #[test]
    fn max_loot_stacks_same_materials() {
        let book = book();
        let max = book.max_loot(CarveTarget::Pack(EnemyKind::Slime));
        let gel_entries = max
            .iter()
            .filter(|(m, _)| *m == MaterialId::SlimeGel)
            .count();
        assert_eq!(gel_entries, 1, "same materials should stack into one entry");
    }

    #[test]
    fn slime_and_bat_have_common_uncommon_rare_parts() {
        let book = book();
        let expected = [
            LootRarity::Common,
            LootRarity::Uncommon,
            LootRarity::Rare,
        ];
        let slime = book.table_for(CarveTarget::Pack(EnemyKind::Slime)).unwrap();
        let bat = book.table_for(CarveTarget::Pack(EnemyKind::Bat)).unwrap();
        assert_eq!(bonus_rarities(slime), expected);
        assert_eq!(bonus_rarities(bat), expected);
    }

    #[test]
    fn rarity_at_splits_default_weights() {
        let weights = RarityWeights::default();
        assert_eq!(rarity_at(weights, 1.0, 0.0), Some(LootRarity::Common));
        assert_eq!(rarity_at(weights, 1.0, 0.5), Some(LootRarity::Common));
        assert_eq!(rarity_at(weights, 1.0, 0.8), Some(LootRarity::Uncommon));
        assert_eq!(rarity_at(weights, 1.0, 0.99), Some(LootRarity::Rare));
    }

    #[test]
    fn zero_rare_multiplier_never_picks_rare() {
        let weights = RarityWeights::default();
        for unit in [0.0, 0.5, 0.94, 0.99, 1.0] {
            assert_ne!(
                rarity_at(weights, 0.0, unit),
                Some(LootRarity::Rare),
                "unit roll {unit} must not be Rare when multiplier is 0"
            );
        }
    }

    #[test]
    fn zero_weights_yield_none() {
        let weights = RarityWeights {
            common: 0,
            uncommon: 0,
            rare: 0,
        };
        assert_eq!(rarity_at(weights, 1.0, 0.5), None);
    }

    #[test]
    fn empty_rarity_bucket_skips_that_bonus_roll() {
        let table = LootTable {
            target: CarveTarget::Pack(EnemyKind::Slime),
            guaranteed: vec![(MaterialId::SlimeGel, 1)],
            bonus: vec![BonusDrop {
                material: MaterialId::SlimeGel,
                amount: 1,
                rarity: LootRarity::Common,
            }],
            rarity_weights: RarityWeights {
                common: 0,
                uncommon: 0,
                rare: 100,
            },
            bonus_rolls: 1,
        };
        let mut rng = StdRng::seed_from_u64(7);
        for _ in 0..20 {
            let drops = roll_table(&table, &mut rng, 1.0);
            assert_eq!(drops, vec![(MaterialId::SlimeGel, 1)]);
        }
    }

    #[test]
    fn missing_table_returns_empty() {
        let book = CarveLootBook {
            tables: Vec::new(),
        };
        let mut rng = StdRng::seed_from_u64(3);
        assert!(book.roll(CarveTarget::KingSlime, &mut rng).is_empty());
    }
}
