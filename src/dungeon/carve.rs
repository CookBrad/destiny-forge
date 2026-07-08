use bevy::prelude::*;

use rand::thread_rng;

use crate::combat::EnemyCorpse;
use crate::combat::PlayerHitFlash;
use crate::core::ProfileDirty;
use crate::graphics::INTERACT_DISTANCE;
use crate::items::Inventory;
use crate::player::Loadout;

use super::carve_loot::{roll_carve_loot, CarveTarget};
use super::enemy::{EnemyKind, KingSlimeBoss};
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
    corpses: Query<
        (
            Entity,
            &Transform,
            Option<&EnemyKind>,
            Option<&KingSlimeBoss>,
        ),
        With<EnemyCorpse>,
    >,
) {
    let Ok((player_transform, hit_flash)) = player.get_single() else {
        carve_state.target = None;
        return;
    };

    if hit_flash.is_some() {
        reset_carve_progress(&mut carve_state, &loadout);
        return;
    }

    let holding = keyboard.pressed(KeyCode::KeyE);
    let nearest = corpses
        .iter()
        .filter(|(_, transform, _, _)| {
            player_transform
                .translation
                .distance(transform.translation)
                <= INTERACT_DISTANCE
        })
        .min_by(|(_, a, _, _), (_, b, _, _)| {
            a.translation
                .distance(player_transform.translation)
                .partial_cmp(&b.translation.distance(player_transform.translation))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    let Some((entity, _, kind, boss)) = nearest else {
        carve_state.target = None;
        return;
    };

    let Some(target) = carve_target(kind, boss) else {
        carve_state.target = None;
        return;
    };

    if !holding {
        reset_carve_progress(&mut carve_state, &loadout);
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

    grant_carve_loot(
        &mut inventory,
        &mut loot_log,
        target,
        &mut thread_rng(),
    );

    commands.entity(entity).try_despawn_recursive();
    reset_carve_progress(&mut carve_state, &loadout);
    profile_dirty.mark();
    info!("Carved {} — materials added to inventory.", target.label());
}

fn carve_target(kind: Option<&EnemyKind>, boss: Option<&KingSlimeBoss>) -> Option<CarveTarget> {
    if boss.is_some() {
        return Some(CarveTarget::KingSlime);
    }
    kind.copied().map(CarveTarget::Pack)
}

fn grant_carve_loot(
    inventory: &mut Inventory,
    loot_log: &mut LootLog,
    target: CarveTarget,
    rng: &mut impl rand::Rng,
) {
    for (material, amount) in roll_carve_loot(target, rng) {
        let leftover = inventory.try_add(material, amount);
        let received = amount.saturating_sub(leftover);
        loot_log.push_carved(material.display_name(), received);
        if leftover > 0 {
            loot_log.push_missed(material.display_name(), leftover);
            warn!("Inventory full — could not store all {material:?}");
        }
    }
}

fn reset_carve_progress(carve_state: &mut CarveState, loadout: &Loadout) {
    carve_state.target = None;
    carve_state.timer = carve_timer(loadout.carve_speed_multiplier());
}

fn carve_timer(speed_multiplier: f32) -> Timer {
    Timer::from_seconds(BASE_CARVE_SECS / speed_multiplier, TimerMode::Once)
}
