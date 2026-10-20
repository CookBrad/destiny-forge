mod actors;
mod props;
mod terrain;

use std::collections::HashSet;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::graphics::DungeonScrollBounds;
use crate::input::{Action, ActionState};
use crate::player::{Loadout, WorldProgress};

use super::enemy::DungeonProgress;
use super::generation::{generate_floor, random_seed};
use super::level::DungeonLayout;
use super::sprites::DungeonArt;

pub use actors::{spawn_enemies, spawn_king_slime, spawn_player};
pub use props::spawn_props;
pub use terrain::{
    spawn_backdrop, spawn_ceiling, spawn_ground, spawn_ladder_exit, spawn_pitfalls,
    spawn_platform,
};

#[derive(Component)]
pub struct DungeonEntity;

#[derive(Component, Clone, Copy)]
pub struct PlatformCollider {
    pub min_x: f32,
    pub max_x: f32,
    pub top_y: f32,
}

#[derive(Component)]
pub struct DungeonExit;

#[derive(Component)]
pub struct Pitfall;

pub fn setup_dungeon(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    loadout: Res<Loadout>,
    world_progress: Res<WorldProgress>,
) {
    setup_dungeon_with_seed(
        &mut commands,
        &asset_server,
        &mut images,
        None,
        &loadout,
        &world_progress,
    );
}

pub fn setup_dungeon_with_seed(
    commands: &mut Commands,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    seed: Option<u64>,
    loadout: &Loadout,
    world_progress: &WorldProgress,
) {
    let art = DungeonArt::load(asset_server, images);
    // Fresh entry gets a random seed; death retry reuses the layout seed.
    let seed = seed.unwrap_or_else(random_seed);
    let floor = generate_floor(seed);

    let mut progress = DungeonProgress::default();
    world_progress.apply_to_dungeon_progress(&mut progress);
    commands.insert_resource(progress);
    commands.insert_resource(DungeonLayout {
        seed,
        floor: floor.clone(),
    });
    commands.insert_resource(DungeonScrollBounds {
        width: floor.width_pixels(),
    });

    spawn_backdrop(commands, &art, &floor);
    spawn_ceiling(commands, &art, &floor);
    spawn_props(commands, &art, &floor);
    for segment in &floor.ground_segments {
        spawn_ground(commands, &art, *segment);
    }
    spawn_pitfalls(commands, &art, &floor.pitfalls);
    for platform in &floor.platforms {
        spawn_platform(commands, &art, *platform);
    }
    spawn_ladder_exit(commands, &art, floor.ladder_tile);
    spawn_player(commands, &art, floor.player_start_x, loadout);
    spawn_enemies(commands, &art, &floor);
    if floor.has_boss {
        spawn_king_slime(commands, &art, floor.boss);
    }

    info!(
        "Loaded dungeon floor (seed {seed}, {} tiles, {} enemies, {} bats, boss={})",
        floor.width_tiles,
        floor.enemies.len(),
        floor.bats.len(),
        floor.has_boss
    );
    commands.insert_resource(art);
}

fn despawn_dungeon(
    commands: &mut Commands,
    entities: &Query<Entity, With<DungeonEntity>>,
    parents: &Query<&Parent>,
) {
    commands.remove_resource::<DungeonArt>();
    commands.remove_resource::<DungeonProgress>();
    commands.remove_resource::<DungeonScrollBounds>();
    commands.remove_resource::<DungeonLayout>();

    let dungeon_entities: HashSet<Entity> = entities.iter().collect();
    let roots: Vec<Entity> = entities
        .iter()
        .filter(|entity| {
            parents
                .get(*entity)
                .ok()
                .is_none_or(|parent| !dungeon_entities.contains(&parent.get()))
        })
        .collect();

    for entity in roots {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn cleanup_dungeon(
    mut commands: Commands,
    entities: Query<Entity, With<DungeonEntity>>,
    parents: Query<&Parent>,
) {
    despawn_dungeon(&mut commands, &entities, &parents);
}

#[derive(SystemParam)]
pub(crate) struct RetryDungeonParams<'w, 's> {
    actions: Res<'w, ActionState>,
    layout: Option<Res<'w, DungeonLayout>>,
    loadout: Res<'w, Loadout>,
    world_progress: Res<'w, WorldProgress>,
    commands: Commands<'w, 's>,
    asset_server: Res<'w, AssetServer>,
    images: ResMut<'w, Assets<Image>>,
    entities: Query<'w, 's, Entity, With<DungeonEntity>>,
    parents: Query<'w, 's, &'static Parent>,
    next_play: ResMut<'w, NextState<crate::core::DungeonPlayState>>,
}

pub fn retry_dungeon(params: RetryDungeonParams) {
    let RetryDungeonParams {
        actions,
        layout,
        loadout,
        world_progress,
        mut commands,
        asset_server,
        mut images,
        entities,
        parents,
        mut next_play,
    } = params;
    if retry_requested(&actions) {
        let seed = layout.as_deref().map(|layout| layout.seed);
        despawn_dungeon(&mut commands, &entities, &parents);
        setup_dungeon_with_seed(
            &mut commands,
            &asset_server,
            &mut images,
            seed,
            &loadout,
            &world_progress,
        );
        next_play.set(crate::core::DungeonPlayState::Running);
    }
}

fn retry_requested(actions: &ActionState) -> bool {
    actions.just_pressed(Action::Confirm) || actions.just_pressed(Action::Jump)
}

#[cfg(test)]
mod retry_input_tests {
    use bevy::prelude::*;

    use crate::input::action_state_after_pressing;

    use super::retry_requested;

    #[test]
    fn enter_or_space_still_requests_a_retry() {
        assert!(retry_requested(&action_state_after_pressing(
            &[KeyCode::Enter],
            &[]
        )));
        assert!(retry_requested(&action_state_after_pressing(
            &[KeyCode::Space],
            &[]
        )));
        assert!(!retry_requested(&action_state_after_pressing(
            &[KeyCode::KeyE],
            &[]
        )));
    }
}
