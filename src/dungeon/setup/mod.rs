mod actors;
mod terrain;

use std::collections::HashSet;

use bevy::prelude::*;

use crate::graphics::DungeonScrollBounds;
use crate::player::{Loadout, WorldProgress};

use super::enemy::DungeonProgress;
use super::generation::{generate_floor, random_seed};
use super::interaction::LadderPrompt;
use super::level::DungeonLayout;
use super::sprites::DungeonArt;

pub use actors::{spawn_enemies, spawn_king_slime, spawn_player};
pub use terrain::{spawn_backdrop, spawn_ground, spawn_ladder_exit, spawn_pitfalls, spawn_platform};

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
    loadout: Res<Loadout>,
    world_progress: Res<WorldProgress>,
) {
    setup_dungeon_with_seed(&mut commands, &asset_server, None, &loadout, &world_progress);
}

pub fn setup_dungeon_with_seed(
    commands: &mut Commands,
    asset_server: &AssetServer,
    seed: Option<u64>,
    loadout: &Loadout,
    world_progress: &WorldProgress,
) {
    let art = DungeonArt::load(asset_server);
    let seed = seed.unwrap_or_else(random_seed);
    let floor = generate_floor(seed);

    commands.init_resource::<LadderPrompt>();
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
    spawn_king_slime(commands, &art, floor.boss);

    info!("Generated dungeon floor (seed {seed}, {} tiles)", floor.width_tiles);
    commands.insert_resource(art);
}

fn despawn_dungeon(
    commands: &mut Commands,
    entities: &Query<Entity, With<DungeonEntity>>,
    parents: &Query<&Parent>,
) {
    commands.remove_resource::<DungeonArt>();
    commands.remove_resource::<LadderPrompt>();
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

pub fn retry_dungeon(
    keyboard: Res<ButtonInput<KeyCode>>,
    layout: Option<Res<DungeonLayout>>,
    loadout: Res<Loadout>,
    world_progress: Res<WorldProgress>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    entities: Query<Entity, With<DungeonEntity>>,
    parents: Query<&Parent>,
    mut next_play: ResMut<NextState<crate::core::DungeonPlayState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        let seed = layout.as_deref().map(|layout| layout.seed);
        despawn_dungeon(&mut commands, &entities, &parents);
        setup_dungeon_with_seed(
            &mut commands,
            &asset_server,
            seed,
            &loadout,
            &world_progress,
        );
        next_play.set(crate::core::DungeonPlayState::Running);
    }
}