use bevy::prelude::*;

use crate::audio::AudioSettings;
use crate::combat::SkillBindings;
use crate::core::{
    save_root_display, ActiveProfile, DungeonPlayState, GameSettings, PlayerProfile, ProfileDirty,
    PROFILE_COUNT, GameState,
};
use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::pause_audio::spawn_pause_audio_controls;
use super::pause_inventory::spawn_pause_inventory_panel;
use super::profile_picker::{select_profile_for_run, ProfilePicker};

#[derive(Component)]
pub struct TitleMenu;

#[derive(Component)]
pub struct PauseMenu;

#[derive(Component)]
pub struct DeathMenu;

#[derive(Component, Clone, Copy)]
pub struct TitleProfileRow {
    pub index: u8,
}

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
                Text::new("Select profile (1-3), then Enter to begin"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.88, 0.38)),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|profiles| {
                    for index in 0..PROFILE_COUNT {
                        let summary = &picker.cards[index as usize];
                        let selected = index == picker.selected;
                        let boss = if summary.boss_cleared { " ✓ boss" } else { "" };
                        let line = format!(
                            "{}Profile {} — {} · {} mats{boss}",
                            if selected { "> " } else { "  " },
                            index + 1,
                            summary.weapon,
                            summary.materials,
                        );
                        profiles.spawn((
                            TitleProfileRow { index },
                            Text::new(line),
                            TextFont {
                                font_size: if selected { 20.0 } else { 17.0 },
                                ..default()
                            },
                            TextColor(if selected {
                                Color::srgb(0.95, 0.9, 0.45)
                            } else {
                                Color::srgb(0.68, 0.72, 0.78)
                            }),
                        ));
                    }
                });

            parent.spawn((
                Text::new(format!("Saves: {}", save_root_display())),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.45, 0.48, 0.52)),
            ));
            parent.spawn((
                Text::new("A/D move  ·  Space jump (2x)  ·  1-9 skills  ·  Hold E carve"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.55, 0.58, 0.64)),
            ));
        });
}

pub fn title_profile_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut picker: ResMut<ProfilePicker>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        picker.selected = 0;
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        picker.selected = 1;
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        picker.selected = 2;
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        picker.selected = picker.selected.saturating_sub(1);
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        picker.selected = (picker.selected + 1).min(PROFILE_COUNT - 1);
    }
}

pub fn sync_title_profile_rows(
    picker: Res<ProfilePicker>,
    mut rows: Query<(&TitleProfileRow, &mut Text, &mut TextColor)>,
) {
    if !picker.is_changed() {
        return;
    }

    for (row, mut text, mut color) in &mut rows {
        let summary = &picker.cards[row.index as usize];
        let selected = row.index == picker.selected;
        let boss = if summary.boss_cleared { " ✓ boss" } else { "" };
        text.0 = format!(
            "{}Profile {} — {} · {} mats{boss}",
            if selected { "> " } else { "  " },
            row.index + 1,
            summary.weapon,
            summary.materials,
        );
        color.0 = if selected {
            Color::srgb(0.95, 0.9, 0.45)
        } else {
            Color::srgb(0.68, 0.72, 0.78)
        };
    }
}

pub fn title_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    picker: Res<ProfilePicker>,
    mut inventory: ResMut<Inventory>,
    mut loadout: ResMut<Loadout>,
    mut progress: ResMut<WorldProgress>,
    mut active: ResMut<ActiveProfile>,
    mut profile: ResMut<PlayerProfile>,
    mut audio: ResMut<AudioSettings>,
    mut bindings: ResMut<SkillBindings>,
    mut global: ResMut<GameSettings>,
    mut profile_dirty: ResMut<ProfileDirty>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        select_profile_for_run(
            &picker,
            &mut inventory,
            &mut loadout,
            &mut progress,
            &mut audio,
            &mut bindings,
            &mut active,
            &mut profile,
            &mut global,
            &mut profile_dirty,
        );
        next_state.set(GameState::Dungeon);
    }
}

pub fn cleanup_title_menu(mut commands: Commands, menus: Query<Entity, With<TitleMenu>>) {
    for entity in &menus {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn spawn_pause_menu(
    mut commands: Commands,
    settings: Res<AudioSettings>,
    inventory: Res<Inventory>,
) {
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
            spawn_pause_inventory_panel(parent, &inventory);
            parent.spawn((
                Text::new("Esc — Resume"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.82, 0.88)),
            ));
            parent.spawn((
                Text::new("Q — Quit to title"),
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
        commands.entity(entity).despawn_recursive();
    }
}

pub fn open_pause_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_play: ResMut<NextState<DungeonPlayState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_play.set(DungeonPlayState::Paused);
    }
}

pub fn pause_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_play: ResMut<NextState<DungeonPlayState>>,
    mut next_game: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_play.set(DungeonPlayState::Running);
    }

    if keyboard.just_pressed(KeyCode::KeyQ) {
        next_game.set(GameState::Title);
    }
}

pub fn pause_game_time(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

pub fn resume_game_time(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
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
                Text::new("Q — Quit to title"),
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
        commands.entity(entity).despawn_recursive();
    }
}

pub fn death_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_game: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyQ) {
        next_game.set(GameState::Title);
    }
}