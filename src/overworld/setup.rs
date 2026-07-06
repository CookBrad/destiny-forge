use bevy::prelude::*;

use crate::graphics::{center_on_surface, world_transform, TILE};

use super::layout::{spawn_homestead, tile_center, OverworldLayout, WORLD_WIDTH};
use super::movement::{ExplorationMap, MapTransitionCooldown, OverworldPlayer};
use super::sprites::{OverworldArt, PLAYER_SPRITE_HEIGHT};

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq)]
pub enum OverworldEntry {
    #[default]
    Yard,
    ForestTrail,
}

#[derive(Component)]
pub struct OverworldHud;

pub fn setup_overworld(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    entry: Option<Res<OverworldEntry>>,
) {
    let art = OverworldArt::load(&asset_server, &mut atlas_layouts);
    let layout = OverworldLayout::homestead();
    let spawn = entry.map(|entry| *entry).unwrap_or_default();

    spawn_homestead(&mut commands, &art, &layout);
    spawn_overworld_player(&mut commands, &art, spawn);

    commands.insert_resource(ExplorationMap {
        solids: layout.solids.clone(),
        world_width: super::layout::WORLD_WIDTH,
        world_height: super::layout::WORLD_HEIGHT,
    });
    commands.insert_resource(MapTransitionCooldown::default());
    commands.insert_resource(art);
    commands.insert_resource(layout);
    commands.remove_resource::<OverworldEntry>();
}

fn spawn_overworld_player(commands: &mut Commands, art: &OverworldArt, entry: OverworldEntry) {
    let start = match entry {
        OverworldEntry::Yard => Vec2::new(WORLD_WIDTH * 0.5, TILE * 12.0),
        OverworldEntry::ForestTrail => tile_center(3, 36),
    };
    let y = center_on_surface(start.y, PLAYER_SPRITE_HEIGHT);

    commands.spawn((
        Sprite {
            image: art.player.idle[0].clone(),
            ..default()
        },
        world_transform(Vec2::new(start.x, y), 5.0),
        OverworldPlayer,
        super::movement::OverworldVelocity::default(),
    ));
}

pub fn spawn_overworld_hud(mut commands: Commands) {
    commands
        .spawn((
            OverworldHud,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
        ))
        .with_children(|hud| {
            hud.spawn((
                OverworldZoneLabel,
                Text::new("Homestead"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.94, 0.82)),
            ));
            hud.spawn((
                OverworldPromptLabel,
                Text::new("WASD move  ·  E interact  ·  Esc title"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.68, 0.72, 0.78)),
            ));
        });
}

#[derive(Component)]
pub struct OverworldZoneLabel;

#[derive(Component)]
pub struct OverworldPromptLabel;

pub fn cleanup_overworld(
    mut commands: Commands,
    entities: Query<Entity, With<super::layout::OverworldEntity>>,
    players: Query<Entity, With<OverworldPlayer>>,
    hud: Query<Entity, With<OverworldHud>>,
) {
    for entity in entities.iter().chain(players.iter()).chain(hud.iter()) {
        commands.entity(entity).try_despawn_recursive();
    }
    commands.remove_resource::<OverworldArt>();
    commands.remove_resource::<OverworldLayout>();
    commands.remove_resource::<ExplorationMap>();
    commands.remove_resource::<MapTransitionCooldown>();
}