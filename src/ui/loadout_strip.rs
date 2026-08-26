use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;

use crate::combat::WeaponKind;
use crate::core::{GameState, ProfileDirty};
use crate::player::{weapon_kind_label, ArmorKind, Loadout};

use super::inventory_window::InventoryWindowOpen;

const HINT: Color = Color::srgb(0.52, 0.5, 0.46);
const LABEL: Color = Color::srgb(0.9, 0.88, 0.84);
const BUTTON_BG: Color = Color::srgb(0.18, 0.14, 0.1);
const BUTTON_BORDER: Color = Color::srgb(0.42, 0.32, 0.18);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadoutSwapAccess {
    Hub,
    Locked,
}

impl LoadoutSwapAccess {
    pub fn from_game_state(state: &GameState) -> Self {
        match state {
            GameState::Overworld => Self::Hub,
            _ => Self::Locked,
        }
    }
}

#[derive(Component)]
pub struct EquippedWeaponLabel;

#[derive(Component)]
pub struct StashHintLabel;

#[derive(Component, Clone, Copy)]
pub struct StashWeaponButton {
    pub weapon: WeaponKind,
}

#[derive(Component, Clone, Copy)]
pub struct StashArmorButton {
    pub kind: ArmorKind,
}

pub fn spawn_loadout_strip(
    parent: &mut ChildBuilder<'_>,
    loadout: &Loadout,
    access: LoadoutSwapAccess,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            width: Val::Percent(100.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
            border: UiRect::top(Val::Px(2.0)),
            ..default()
        })
        .with_children(|strip| {
            spawn_equipped_row(strip, loadout);
            spawn_stash_row(strip, loadout, access);
        });
}

fn spawn_equipped_row(parent: &mut ChildBuilder<'_>, loadout: &Loadout) {
    parent.spawn((
        EquippedWeaponLabel,
        Text::new(equipped_text(loadout)),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(LABEL),
    ));
}

fn spawn_stash_row(
    parent: &mut ChildBuilder<'_>,
    loadout: &Loadout,
    access: LoadoutSwapAccess,
) {
    parent.spawn((
        StashHintLabel,
        Text::new(hint_text(loadout, access)),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(HINT),
    ));

    if access != LoadoutSwapAccess::Hub {
        return;
    }

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(6.0),
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|row| {
            for weapon in &loadout.stash.weapons {
                spawn_weapon_button(row, *weapon);
            }
            for kind in &loadout.stash.armor {
                spawn_armor_button(row, *kind);
            }
        });
}

fn spawn_weapon_button(parent: &mut ChildBuilder<'_>, weapon: WeaponKind) {
    parent
        .spawn((
            Button,
            StashWeaponButton { weapon },
            Node {
                height: Val::Px(24.0),
                padding: UiRect::horizontal(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(BUTTON_BG),
            BorderColor(BUTTON_BORDER),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(weapon_kind_label(weapon)),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(LABEL),
            ));
        });
}

fn spawn_armor_button(parent: &mut ChildBuilder<'_>, kind: ArmorKind) {
    parent
        .spawn((
            Button,
            StashArmorButton { kind },
            Node {
                height: Val::Px(24.0),
                padding: UiRect::horizontal(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(BUTTON_BG),
            BorderColor(BUTTON_BORDER),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(kind.label()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(LABEL),
            ));
        });
}

fn equipped_text(loadout: &Loadout) -> String {
    format!("Equipped: {}", loadout.weapon_label())
}

fn hint_text(loadout: &Loadout, access: LoadoutSwapAccess) -> String {
    match access {
        LoadoutSwapAccess::Locked => "Swap loadout at the homestead.".to_string(),
        LoadoutSwapAccess::Hub if loadout.stash.weapons.is_empty() && loadout.stash.armor.is_empty() => {
            "Stash is empty. Forging an alternate stores the old piece.".to_string()
        }
        LoadoutSwapAccess::Hub => {
            "Click a stashed piece to equip it  ·  [ ] cycle weapons".to_string()
        }
    }
}

pub fn handle_stash_weapon_click(
    interactions: Query<(&Interaction, &StashWeaponButton), (Changed<Interaction>, With<Button>)>,
    mut loadout: ResMut<Loadout>,
    mut dirty: ResMut<ProfileDirty>,
) {
    for (interaction, button) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if loadout.swap_to_stashed_weapon(button.weapon) {
            dirty.mark();
        }
    }
}

pub fn handle_stash_armor_click(
    interactions: Query<(&Interaction, &StashArmorButton), (Changed<Interaction>, With<Button>)>,
    mut loadout: ResMut<Loadout>,
    mut dirty: ResMut<ProfileDirty>,
) {
    for (interaction, button) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if loadout.swap_to_stashed_armor(button.kind) {
            dirty.mark();
        }
    }
}

pub fn handle_loadout_swap_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    game: Res<State<GameState>>,
    mut loadout: ResMut<Loadout>,
    mut dirty: ResMut<ProfileDirty>,
) {
    if LoadoutSwapAccess::from_game_state(game.get()) != LoadoutSwapAccess::Hub {
        return;
    }
    if loadout.stash.weapons.is_empty() {
        return;
    }

    let forward = keyboard.just_pressed(KeyCode::BracketRight);
    let back = keyboard.just_pressed(KeyCode::BracketLeft);
    if !forward && !back {
        return;
    }

    let Some(weapon) = next_stashed_weapon(&loadout, forward) else {
        return;
    };
    if loadout.swap_to_stashed_weapon(weapon) {
        dirty.mark();
    }
}

fn next_stashed_weapon(loadout: &Loadout, forward: bool) -> Option<WeaponKind> {
    let weapons = &loadout.stash.weapons;
    if weapons.is_empty() {
        return None;
    }
    if forward {
        weapons.first().copied()
    } else {
        weapons.last().copied()
    }
}

pub fn sync_loadout_strip(
    loadout: Res<Loadout>,
    game: Res<State<GameState>>,
    open: Res<InventoryWindowOpen>,
    mut equipped: Query<&mut Text, (With<EquippedWeaponLabel>, Without<StashHintLabel>)>,
    mut hint: Query<&mut Text, (With<StashHintLabel>, Without<EquippedWeaponLabel>)>,
) {
    if !open.0 || !loadout.is_changed() {
        return;
    }
    let access = LoadoutSwapAccess::from_game_state(game.get());
    if let Ok(mut text) = equipped.get_single_mut() {
        text.0 = equipped_text(&loadout);
    }
    if let Ok(mut text) = hint.get_single_mut() {
        text.0 = hint_text(&loadout, access);
    }
}
