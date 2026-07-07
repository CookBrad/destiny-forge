use bevy::prelude::*;

use crate::audio::AudioSettings;
use crate::core::{save_root_display, DungeonPlayState, GameState};
use super::inventory_window::InventoryWindowOpen;
use super::pause_audio::spawn_pause_audio_controls;
use super::profile_picker::ProfilePicker;
use super::title_profiles::spawn_title_profile_cards;

#[derive(Component)]
pub struct TitleMenu;

#[derive(Component)]
pub struct TitleHintLabel;

#[derive(Component)]
pub struct PauseMenu;

#[derive(Component)]
pub struct DeathMenu;

pub fn spawn_title_menu(mut commands: Commands, picker: Res<ProfilePicker>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(18.0),
                padding: UiRect::all(Val::Px(32.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.03, 0.08, 0.94)),
            TitleMenu,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Destiny Forge"),
                TextFont {
                    font_size: 58.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.96, 0.68)),
            ));
            parent.spawn((
                Text::new("Forge your fate in the depths below."),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.76, 0.8)),
            ));
            parent.spawn((
                TitleHintLabel,
                Text::new("Click a profile to play · Rename to customize · 1-3 quick start"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.88, 0.38)),
            ));

            spawn_title_profile_cards(parent, &picker);

            parent.spawn((
                Text::new(format!("Saves: {}", save_root_display())),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.45, 0.48, 0.52)),
            ));
            parent.spawn((
                Text::new("Homestead: WASD explore  ·  Dungeon: A/D  ·  Space jump  ·  E interact"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.55, 0.58, 0.64)),
            ));
        });
}

pub fn sync_title_hint(
    rename: Res<super::title_profiles::ProfileRenameState>,
    mut hints: Query<&mut Text, With<TitleHintLabel>>,
) {
    if !rename.is_changed() {
        return;
    }

    let message = if rename.active.is_some() {
        "Type a name · Enter to save · Esc to cancel"
    } else {
        "Click a profile to play · Rename to customize · 1-3 quick start"
    };

    for mut text in &mut hints {
        text.0 = message.to_string();
    }
}

pub fn cleanup_title_menu(mut commands: Commands, menus: Query<Entity, With<TitleMenu>>) {
    for entity in &menus {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn spawn_pause_menu(mut commands: Commands, settings: Res<AudioSettings>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(22.0),
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.05, 0.72)),
            PauseMenu,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Paused"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.92, 0.95)),
            ));
            spawn_pause_audio_controls(parent, &settings);
            parent.spawn((
                Text::new("I — Inventory  ·  Esc — Resume"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.82, 0.88)),
            ));
            parent.spawn((
                Text::new("Q — Return to homestead"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.82, 0.88)),
            ));
        });
}

pub fn cleanup_pause_menu(mut commands: Commands, menus: Query<Entity, With<PauseMenu>>) {
    for entity in &menus {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn open_pause_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<InventoryWindowOpen>,
    mut next_play: ResMut<NextState<DungeonPlayState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && !inventory.0 {
        next_play.set(DungeonPlayState::Paused);
    }
}

pub fn pause_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<InventoryWindowOpen>,
    mut next_play: ResMut<NextState<DungeonPlayState>>,
    mut next_game: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && !inventory.0 {
        next_play.set(DungeonPlayState::Running);
    }

    if keyboard.just_pressed(KeyCode::KeyQ) {
        next_game.set(GameState::Overworld);
    }
}

pub fn pause_game_time(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

pub fn resume_game_time(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

pub fn set_title_clear_color(mut clear: ResMut<ClearColor>) {
    clear.0 = Color::srgb(0.08, 0.07, 0.1);
}

pub fn ensure_time_running(mut time: ResMut<Time<Virtual>>) {
    if time.is_paused() {
        time.unpause();
    }
}

pub fn spawn_death_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.02, 0.04, 0.82)),
            DeathMenu,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("You Died"),
                TextFont {
                    font_size: 52.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.35, 0.38)),
            ));
            parent.spawn((
                Text::new("Enter or Space — Try again"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.82, 0.88)),
            ));
            parent.spawn((
                Text::new("Q — Return to homestead"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.82, 0.88)),
            ));
        });
}

pub fn cleanup_death_menu(mut commands: Commands, menus: Query<Entity, With<DeathMenu>>) {
    for entity in &menus {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn death_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_game: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyQ) {
        next_game.set(GameState::Overworld);
    }
}