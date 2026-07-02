use bevy::prelude::*;

use crate::combat::{EquippedWeapon, Health, PlayerAttack};
use crate::graphics::{
    player_display_size, player_half_extents, scaled_size, scaled_transform, DUNGEON_FLOOR_Y, TILE,
};

use super::animation::PlayerAnimation;
use super::enemy::{BatEnemy, Patrol, SlimeEnemy};
use super::interaction::LadderPrompt;
use super::level::FloorOne;
use super::movement::{DungeonPlayer, PlayerVelocity};
use super::sprites::{player_frame_rect, DungeonArt};

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

pub fn setup_dungeon(mut commands: Commands, asset_server: Res<AssetServer>) {
    let art = DungeonArt::load(&asset_server);
    commands.init_resource::<LadderPrompt>();

    spawn_backdrop(&mut commands, &art);
    spawn_ground(&mut commands, &art, FloorOne::GROUND);
    for platform in FloorOne::PLATFORMS {
        spawn_platform(&mut commands, &art, *platform);
    }
    spawn_ladder_exit(&mut commands, &art);
    spawn_player(&mut commands, &art);
    for slime in FloorOne::SLIMES {
        spawn_slime(&mut commands, &art, slime.x, slime.top_y);
    }
    for bat in FloorOne::BATS {
        spawn_bat(&mut commands, &art, bat.x, bat.top_y);
    }

    commands.insert_resource(art);
}

fn spawn_backdrop(commands: &mut Commands, art: &DungeonArt) {
    let wall = art.wall.clone();
    for row in 0..6 {
        for column in 0..24 {
            let position = Vec2::new(column as f32 * TILE, row as f32 * TILE);
            commands.spawn((
                Sprite {
                    image: wall.clone(),
                    ..default()
                },
                scaled_transform(position, 0.0),
                DungeonEntity,
            ));
        }
    }
}

fn spawn_ground(commands: &mut Commands, art: &DungeonArt, spec: super::level::PlatformSpec) {
    spawn_platform_tiles(commands, art, spec, true);
}

fn spawn_platform(commands: &mut Commands, art: &DungeonArt, spec: super::level::PlatformSpec) {
    spawn_platform_tiles(commands, art, spec, false);
}

fn spawn_platform_tiles(
    commands: &mut Commands,
    art: &DungeonArt,
    spec: super::level::PlatformSpec,
    ground: bool,
) {
    let texture = if ground {
        art.floor_ground.clone()
    } else {
        art.floor_platform.clone()
    };

    let width = spec.width_tiles as f32 * TILE;
    let collider = PlatformCollider {
        min_x: spec.left,
        max_x: spec.left + width,
        top_y: spec.top_y,
    };

    for tile in 0..spec.width_tiles {
        let x = spec.left + tile as f32 * TILE + TILE * 0.5;
        let y = spec.top_y - TILE * 0.5;
        commands.spawn((
            Sprite {
                image: texture.clone(),
                ..default()
            },
            scaled_transform(Vec2::new(x, y), 1.0),
            collider,
            DungeonEntity,
        ));
    }
}

fn spawn_ladder_exit(commands: &mut Commands, art: &DungeonArt) {
    let x = FloorOne::LADDER_TILE as f32 * TILE + TILE * 0.5;
    let y = DUNGEON_FLOOR_Y - TILE * 0.5;

    commands.spawn((
        Sprite {
            image: art.floor_ladder.clone(),
            ..default()
        },
        scaled_transform(Vec2::new(x, y), 1.0),
        PlatformCollider {
            min_x: x - TILE * 0.5,
            max_x: x + TILE * 0.5,
            top_y: DUNGEON_FLOOR_Y,
        },
        DungeonExit,
        DungeonEntity,
    ));
}

fn spawn_player(commands: &mut Commands, art: &DungeonArt) {
    let half = player_half_extents();
    let start = Vec2::new(FloorOne::PLAYER_START_X, DUNGEON_FLOOR_Y + half.y);

    commands.spawn((
        Sprite {
            image: art.player_idle.clone(),
            rect: Some(player_frame_rect(0)),
            custom_size: Some(scaled_size(player_display_size())),
            ..default()
        },
        scaled_transform(start, 10.0),
        DungeonPlayer,
        PlayerVelocity::default(),
        PlayerAnimation::default(),
        EquippedWeapon::default(),
        PlayerAttack::inactive(),
        DungeonEntity,
    ));
}

fn spawn_slime(commands: &mut Commands, art: &DungeonArt, x: f32, top_y: f32) {
    let half_height = TILE * 0.5;

    commands.spawn((
        Sprite {
            image: art.slime.clone(),
            ..default()
        },
        scaled_transform(Vec2::new(x, top_y + half_height), 5.0),
        SlimeEnemy,
        Health::new(30.0),
        Patrol::between(x - 2.0 * TILE, x + 2.0 * TILE, 35.0),
        DungeonEntity,
    ));
}

fn spawn_bat(commands: &mut Commands, art: &DungeonArt, x: f32, top_y: f32) {
    let hover_y = top_y + 3.0 * TILE;

    commands.spawn((
        Sprite {
            image: art.bat.clone(),
            ..default()
        },
        scaled_transform(Vec2::new(x, hover_y), 5.0),
        BatEnemy,
        Health::new(20.0),
        Patrol::between(x - TILE, x + TILE, 50.0),
        DungeonEntity,
    ));
}

pub fn cleanup_dungeon(
    mut commands: Commands,
    entities: Query<Entity, With<DungeonEntity>>,
) {
    commands.remove_resource::<DungeonArt>();
    commands.remove_resource::<LadderPrompt>();
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}