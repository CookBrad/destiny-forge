use bevy::prelude::*;

use crate::core::{DungeonPlayState, GameState};

#[derive(Component)]
pub struct TitleMenu;

#[derive(Component)]
pub struct PauseMenu;

#[derive(Component)]
pub struct DeathMenu;

pub fn spawn_title_menu(mut commands: Commands) {
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
                Text::new("Press Enter or Space to begin"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.88, 0.38)),
            ));
            parent.spawn((
                Text::new("A/D move  ·  Space jump  ·  1 attack  ·  2 block  ·  3 charge  ·  4 spin"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.55, 0.58, 0.64)),
            ));
        });
}

pub fn title_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Dungeon);
    }
}

pub fn cleanup_title_menu(mut commands: Commands, menus: Query<Entity, With<TitleMenu>>) {
    for entity in &menus {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn spawn_pause_menu(mut commands: Commands) {
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