use bevy::prelude::*;

use crate::combat::EnemyCorpse;
use crate::combat::PlayerHitFlash;
use crate::core::ProfileDirty;
use crate::graphics::INTERACT_DISTANCE;
use crate::items::{Inventory, MaterialId};
use crate::player::Loadout;

use super::enemy::EnemyKind;
use super::movement::DungeonPlayer;

const BASE_CARVE_SECS: f32 = 2.0;

#[derive(Resource, Default)]
pub struct CarveState {
    pub target: Option<Entity>,
    pub timer: Timer,
}

/// Pending loot lines for the dungeon loot log UI.
#[derive(Resource, Default)]
pub struct LootLog {
    pub pending: Vec<LootLogEntry>,
}

#[derive(Clone, Debug)]
pub struct LootLogEntry {
    pub text: String,
}

impl LootLog {
    pub fn push_carved(&mut self, display_name: &str, amount: u32) {
        if amount == 0 {
            return;
        }
        self.pending.push(LootLogEntry {
            text: format!("Carved {amount}× {display_name}"),
        });
    }

    pub fn push_missed(&mut self, display_name: &str, amount: u32) {
        if amount == 0 {
            return;
        }
        self.pending.push(LootLogEntry {
            text: format!("Inventory full — lost {amount}× {display_name}"),
        });
    }
}

impl EnemyKind {
    pub fn carve_loot(self) -> &'static [(MaterialId, u32)] {
        match self {
            Self::Slime => &[
                (MaterialId::SlimeGel, 3),
                (MaterialId::SlimeCore, 1),
                (MaterialId::IronScrap, 1),
            ],
            Self::Bat => &[(MaterialId::LeatherWing, 1), (MaterialId::Fang, 1)],
            Self::Goblin => &[(MaterialId::Fang, 1), (MaterialId::IronScrap, 1)],
            Self::Skeleton => &[(MaterialId::IronScrap, 2)],
            Self::Zombie => &[(MaterialId::IronScrap, 1), (MaterialId::SlimeGel, 1)],
        }
    }
}

pub fn carve_corpses(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    loadout: Res<Loadout>,
    mut inventory: ResMut<Inventory>,
    mut profile_dirty: ResMut<ProfileDirty>,
    mut carve_state: ResMut<CarveState>,
    mut loot_log: ResMut<LootLog>,
    mut commands: Commands,
    player: Query<(&Transform, Option<&PlayerHitFlash>), With<DungeonPlayer>>,
    corpses: Query<(Entity, &Transform, &EnemyKind), With<EnemyCorpse>>,
) {
    let Ok((player_transform, hit_flash)) = player.get_single() else {
        carve_state.target = None;
        return;
    };

    if hit_flash.is_some() {
        carve_state.target = None;
        carve_state.timer = carve_timer(loadout.carve_speed_multiplier());
        return;
    }

    let holding = keyboard.pressed(KeyCode::KeyE);
    let nearest = corpses
        .iter()
        .filter(|(_, transform, _)| {
            player_transform
                .translation
                .distance(transform.translation)
                <= INTERACT_DISTANCE
        })
        .min_by(|(_, a, _), (_, b, _)| {
            a.translation
                .distance(player_transform.translation)
                .partial_cmp(&b.translation.distance(player_transform.translation))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    let Some((entity, _, kind)) = nearest else {
        carve_state.target = None;
        return;
    };

    if !holding {
        carve_state.target = None;
        carve_state.timer = carve_timer(loadout.carve_speed_multiplier());
        return;
    }

    if carve_state.target != Some(entity) {
        carve_state.target = Some(entity);
        carve_state.timer = carve_timer(loadout.carve_speed_multiplier());
    }

    carve_state.timer.tick(time.delta());
    if !carve_state.timer.just_finished() {
        return;
    }

    for (material, amount) in kind.carve_loot() {
        let leftover = inventory.try_add(*material, *amount);
        let received = amount.saturating_sub(leftover);
        loot_log.push_carved(material.display_name(), received);
        if leftover > 0 {
            loot_log.push_missed(material.display_name(), leftover);
            warn!("Inventory full — could not store all {material:?}");
        }
    }

    commands.entity(entity).try_despawn_recursive();
    carve_state.target = None;
    carve_state.timer = carve_timer(loadout.carve_speed_multiplier());
    profile_dirty.mark();
    info!("Carved {} — materials added to inventory.", kind_debug(*kind));
}

fn carve_timer(speed_multiplier: f32) -> Timer {
    Timer::from_seconds(BASE_CARVE_SECS / speed_multiplier, TimerMode::Once)
}

fn kind_debug(kind: EnemyKind) -> &'static str {
    match kind {
        EnemyKind::Slime => "slime",
        EnemyKind::Bat => "bat",
        EnemyKind::Goblin => "goblin",
        EnemyKind::Skeleton => "skeleton",
        EnemyKind::Zombie => "zombie",
    }
}
