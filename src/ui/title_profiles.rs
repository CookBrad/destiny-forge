use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;

use crate::audio::AudioSettings;
use crate::combat::SkillBindings;
use crate::core::{
    rename_profile_on_disk, sanitize_profile_name, ActiveProfile, GameSettings, GameState,
    PlayerProfile, ProfileDirty, PROFILE_COUNT,
};
use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::profile_picker::{begin_profile_run, ProfilePicker};

const CARD_WIDTH: f32 = 420.0;
const CARD_HEIGHT: f32 = 72.0;

#[derive(Component)]
pub struct TitleProfileContainer;

#[derive(Component, Clone, Copy)]
pub struct TitleProfileCard {
    pub index: u8,
}

#[derive(Component, Clone, Copy)]
pub struct TitleProfileCardName {
    pub index: u8,
}

#[derive(Component, Clone, Copy)]
pub struct TitleProfileCardDetails {
    pub index: u8,
}

#[derive(Component, Clone, Copy)]
pub struct TitleProfileRenameButton {
    pub index: u8,
}

#[derive(Resource, Default)]
pub struct ProfileRenameState {
    pub active: Option<ProfileRenameSession>,
}

#[derive(Clone, Debug)]
pub struct ProfileRenameSession {
    pub index: u8,
    pub buffer: String,
}

pub fn spawn_title_profile_cards(parent: &mut ChildBuilder<'_>, picker: &ProfilePicker) {
    parent
        .spawn((
            TitleProfileContainer,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|profiles| {
            for index in 0..PROFILE_COUNT {
                let summary = &picker.cards[index as usize];
                spawn_profile_card(profiles, index, summary);
            }
        });
}

fn spawn_profile_card(
    parent: &mut ChildBuilder<'_>,
    index: u8,
    summary: &super::profile_picker::ProfileCardSummary,
) {
    parent
        .spawn(Node {
            width: Val::Px(CARD_WIDTH),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Button,
                TitleProfileCard { index },
                Node {
                    flex_grow: 1.0,
                    height: Val::Px(CARD_HEIGHT),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.11, 0.16, 0.95)),
                BorderColor(Color::srgba(0.3, 0.34, 0.42, 0.9)),
            ))
            .with_children(|card| {
                card.spawn((
                    TitleProfileCardName { index },
                    Text::new(&summary.name),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.92, 0.55)),
                ));
                card.spawn((
                    TitleProfileCardDetails { index },
                    Text::new(card_details(summary)),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.68, 0.72, 0.78)),
                ));
            });

            row.spawn((
                Button,
                TitleProfileRenameButton { index },
                Node {
                    min_width: Val::Px(72.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.14, 0.16, 0.22, 0.95)),
                BorderColor(Color::srgba(0.35, 0.38, 0.46, 0.9)),
            ))
            .with_children(|rename| {
                rename.spawn((
                    Text::new("Rename"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.86, 0.92)),
                ));
            });
        });
}

fn card_details(summary: &super::profile_picker::ProfileCardSummary) -> String {
    let boss = if summary.boss_cleared { " · ✓ boss" } else { "" };
    format!("{} · {} mats{boss}", summary.weapon, summary.materials)
}

pub fn handle_title_profile_card_clicks(
    rename: Res<ProfileRenameState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut interactions: Query<
        (&Interaction, &TitleProfileCard),
        (Changed<Interaction>, With<Button>),
    >,
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
    if rename.active.is_some() {
        return;
    }

    for (interaction, card) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        begin_profile_run(
            card.index,
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
        next_state.set(GameState::Overworld);
    }
}

pub fn handle_title_profile_rename_clicks(
    mut rename: ResMut<ProfileRenameState>,
    picker: Res<ProfilePicker>,
    mut interactions: Query<
        (&Interaction, &TitleProfileRenameButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, button) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let summary = &picker.cards[button.index as usize];
        rename.active = Some(ProfileRenameSession {
            index: button.index,
            buffer: summary.name.clone(),
        });
    }
}

pub fn handle_profile_rename_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut rename: ResMut<ProfileRenameState>,
    mut picker: ResMut<ProfilePicker>,
    active: Res<ActiveProfile>,
    mut profile: ResMut<PlayerProfile>,
) {
    let Some(session) = rename.active.as_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Escape) {
        rename.active = None;
        return;
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        let index = session.index;
        let buffer = session.buffer.clone();
        let saved = rename_profile_on_disk(index, buffer);
        *picker = ProfilePicker::refresh();
        if active.index() == index {
            profile.name = saved.name;
        }
        rename.active = None;
        return;
    }

    if keyboard.just_pressed(KeyCode::Backspace) {
        session.buffer.pop();
        return;
    }

    for key in keyboard.get_just_pressed() {
        let Some(ch) = key_to_char(*key) else {
            continue;
        };
        let mut next = session.buffer.clone();
        next.push(ch);
        session.buffer = sanitize_profile_name(next);
    }
}

pub fn handle_title_profile_keyboard_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    rename: Res<ProfileRenameState>,
    mut next_state: ResMut<NextState<GameState>>,
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
    if rename.active.is_some() {
        return;
    }

    let index = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(2)
    } else {
        None
    };

    let Some(index) = index else {
        return;
    };

    begin_profile_run(
        index,
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
    next_state.set(GameState::Overworld);
}

pub fn sync_title_profile_cards(
    picker: Res<ProfilePicker>,
    rename: Res<ProfileRenameState>,
    mut cards: Query<
        (
            &TitleProfileCard,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<TitleProfileCard>,
    >,
    mut names: Query<
        (&TitleProfileCardName, &mut Text, &mut TextColor),
        Without<TitleProfileCardDetails>,
    >,
    mut details: Query<
        (&TitleProfileCardDetails, &mut Text),
        Without<TitleProfileCardName>,
    >,
    mut rename_buttons: Query<
        (&TitleProfileRenameButton, &Interaction, &mut BackgroundColor),
        Without<TitleProfileCard>,
    >,
) {
    for (card, interaction, mut bg, mut border) in &mut cards {
        apply_card_highlight(card.index, rename.active.as_ref(), interaction, &mut bg, &mut border);
    }

    for (name_label, mut text, mut color) in &mut names {
        let summary = &picker.cards[name_label.index as usize];
        let renaming = rename
            .active
            .as_ref()
            .is_some_and(|session| session.index == name_label.index);

        if renaming {
            let buffer = &rename.active.as_ref().unwrap().buffer;
            text.0 = if buffer.is_empty() {
                "▌".to_string()
            } else {
                format!("{buffer}▌")
            };
            color.0 = Color::srgb(0.98, 0.94, 0.62);
        } else {
            text.0 = summary.name.clone();
            color.0 = Color::srgb(0.95, 0.92, 0.55);
        }
    }

    for (details_label, mut text) in &mut details {
        let summary = &picker.cards[details_label.index as usize];
        text.0 = card_details(summary);
    }

    for (button, interaction, mut bg) in &mut rename_buttons {
        apply_rename_highlight(button.index, rename.active.as_ref(), interaction, &mut bg);
    }
}

fn apply_card_highlight(
    index: u8,
    rename: Option<&ProfileRenameSession>,
    interaction: &Interaction,
    bg: &mut BackgroundColor,
    border: &mut BorderColor,
) {
    let renaming = rename.is_some_and(|session| session.index == index);

    if renaming {
        bg.0 = Color::srgba(0.16, 0.18, 0.28, 0.98);
        border.0 = Color::srgb(0.95, 0.82, 0.35);
    } else if matches!(*interaction, Interaction::Hovered | Interaction::Pressed) {
        bg.0 = Color::srgba(0.14, 0.16, 0.24, 0.98);
        border.0 = Color::srgb(0.55, 0.78, 0.95);
    } else {
        bg.0 = Color::srgba(0.1, 0.11, 0.16, 0.95);
        border.0 = Color::srgba(0.3, 0.34, 0.42, 0.9);
    }
}

fn apply_rename_highlight(
    index: u8,
    rename: Option<&ProfileRenameSession>,
    interaction: &Interaction,
    bg: &mut BackgroundColor,
) {
    let renaming = rename.is_some_and(|session| session.index == index);

    if renaming {
        bg.0 = Color::srgba(0.22, 0.24, 0.32, 0.98);
    } else if matches!(*interaction, Interaction::Hovered | Interaction::Pressed) {
        bg.0 = Color::srgba(0.2, 0.22, 0.3, 0.98);
    } else {
        bg.0 = Color::srgba(0.14, 0.16, 0.22, 0.95);
    }
}

fn key_to_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some('-'),
        KeyCode::Period => Some('.'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        _ => None,
    }
}