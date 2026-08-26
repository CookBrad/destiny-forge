use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;

use crate::combat::WeaponKind;
use crate::core::{GameState, ProfileDirty};
use crate::player::{weapon_kind_label, ArmorKind, Loadout};

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
struct EquippedWeaponLabel;

#[derive(Component)]
struct StashHintLabel;

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
        Text::new(format!("Equipped: {}", loadout.weapon_label())),
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
        Text::new(stash_hint(loadout, access)),
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
    spawn_stash_button(
        parent,
        StashWeaponButton { weapon },
        weapon_kind_label(weapon),
    );
}

fn spawn_armor_button(parent: &mut ChildBuilder<'_>, kind: ArmorKind) {
    spawn_stash_button(parent, StashArmorButton { kind }, kind.label());
}

fn spawn_stash_button(
    parent: &mut ChildBuilder<'_>,
    marker: impl Component,
    label: &'static str,
) {
    parent
        .spawn((
            Button,
            marker,
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
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(LABEL),
            ));
        });
}

fn stash_hint(loadout: &Loadout, access: LoadoutSwapAccess) -> String {
    match access {
        LoadoutSwapAccess::Locked => "Swap loadout at the homestead.".to_string(),
        LoadoutSwapAccess::Hub
            if loadout.stash.weapons.is_empty() && loadout.stash.armor.is_empty() =>
        {
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
        if *interaction == Interaction::Pressed && loadout.swap_to_stashed_weapon(button.weapon) {
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
        if *interaction == Interaction::Pressed && loadout.swap_to_stashed_armor(button.kind) {
            dirty.mark();
        }
    }
}

enum StashCycle {
    First,
    Last,
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

    let cycle = if keyboard.just_pressed(KeyCode::BracketRight) {
        StashCycle::First
    } else if keyboard.just_pressed(KeyCode::BracketLeft) {
        StashCycle::Last
    } else {
        return;
    };

    let Some(weapon) = cycled_stash_weapon(&loadout, cycle) else {
        return;
    };
    if loadout.swap_to_stashed_weapon(weapon) {
        dirty.mark();
    }
}

fn cycled_stash_weapon(loadout: &Loadout, cycle: StashCycle) -> Option<WeaponKind> {
    match cycle {
        StashCycle::First => loadout.stash.weapons.first().copied(),
        StashCycle::Last => loadout.stash.weapons.last().copied(),
    }
}
