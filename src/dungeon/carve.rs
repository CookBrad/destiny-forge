use bevy::prelude::*;

use crate::combat::EnemyCorpse;
use crate::core::ProfileDirty;
use crate::graphics::INTERACT_DISTANCE;
use crate::items::{Inventory, MaterialId};

use super::enemy::EnemyKind;
use super::movement::DungeonPlayer;

const CARVE_SECS: f32 = 2.0;

#[derive(Component)]
pub struct CarvedCorpse;

#[derive(Resource, Default)]
pub struct CarveState {
    pub target: Option<Entity>,
    pub timer: Timer,
}

impl EnemyKind {
    pub fn carve_loot(self) -> &'static [(MaterialId, u32)] {
        match self {
            Self::Slime => &[(MaterialId::SlimeGel, 2), (MaterialId::SlimeCore, 1)],
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
    mut inventory: ResMut<Inventory>,
    mut profile_dirty: ResMut<ProfileDirty>,
    mut carve_state: ResMut<CarveState>,
    mut commands: Commands,
    player: Query<&Transform, With<DungeonPlayer>>,
    corpses: Query<(Entity, &Transform, &EnemyKind), (With<EnemyCorpse>, Without<CarvedCorpse>)>,
) {
    let Ok(player_transform) = player.get_single() else {
        carve_state.target = None;
        return;
    };

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
        carve_state.timer = Timer::from_seconds(CARVE_SECS, TimerMode::Once);
        return;
    }

    if carve_state.target != Some(entity) {
        carve_state.target = Some(entity);
        carve_state.timer = Timer::from_seconds(CARVE_SECS, TimerMode::Once);
    }

    carve_state.timer.tick(time.delta());
    if !carve_state.timer.just_finished() {
        return;
    }

    for (material, amount) in kind.carve_loot() {
        let leftover = inventory.try_add(*material, *amount);
        if leftover > 0 {
            warn!("Inventory full — could not store all {material:?}");
        }
    }

    commands.entity(entity).insert(CarvedCorpse);
    carve_state.target = None;
    profile_dirty.mark();
    info!("Carved {} — materials added to inventory.", kind_debug(*kind));
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