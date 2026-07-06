use bevy::prelude::*;

use crate::graphics::{center_on_surface, scaled_transform, PIXEL_SCALE, TILE};

use super::layout::{spawn_homestead, OverworldLayout, WORLD_WIDTH};
use super::movement::OverworldPlayer;
use super::sprites::{OverworldArt, PLAYER_SPRITE_HEIGHT};

#[derive(Component)]
pub struct OverworldHud;

pub fn setup_overworld(mut commands: Commands, asset_server: Res<AssetServer>) {
    let art = OverworldArt::load(&asset_server);
    let layout = OverworldLayout::homestead();

    spawn_homestead(&mut commands, &art, &layout);
    spawn_overworld_player(&mut commands, &art);

    commands.insert_resource(art);
    commands.insert_resource(layout);
}

fn spawn_overworld_player(commands: &mut Commands, art: &OverworldArt) {
    let start = Vec2::new(WORLD_WIDTH * 0.5, TILE * 12.0);
    let y = center_on_surface(start.y, PLAYER_SPRITE_HEIGHT);

    commands.spawn((
        Sprite {
            image: art.player.idle[0].clone(),
            ..default()
        },
        scaled_transform(Vec2::new(start.x, y), 5.0),
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
}