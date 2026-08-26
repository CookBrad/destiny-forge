//! Escape on homestead / forest / lake: confirm before returning to the title screen.

use bevy::prelude::*;

use crate::core::GameState;
use crate::fishing::ActiveCast;
use crate::ui::forge_window::ForgeWindowOpen;
use crate::ui::inventory_window::InventoryWindowOpen;

#[derive(Resource, Default, Debug)]
pub struct ExitConfirmOpen(pub bool);

#[derive(Component)]
pub struct ExitConfirmRoot;

#[derive(Component)]
pub struct ExitConfirmYes;

#[derive(Component)]
pub struct ExitConfirmNo;

pub fn exit_confirm_closed(open: Res<ExitConfirmOpen>) -> bool {
    !open.0
}

pub fn open_exit_confirm(
    commands: &mut Commands,
    open: &mut ExitConfirmOpen,
    time: &mut Time<Virtual>,
) {
    if open.0 {
        return;
    }
    open.0 = true;
    spawn_exit_confirm(commands);
    if !time.is_paused() {
        time.pause();
    }
}

pub fn close_exit_confirm(
    commands: &mut Commands,
    open: &mut ExitConfirmOpen,
    roots: &Query<Entity, With<ExitConfirmRoot>>,
    time: &mut Time<Virtual>,
) {
    open.0 = false;
    for entity in roots.iter() {
        commands.entity(entity).try_despawn_recursive();
    }
    if time.is_paused() {
        time.unpause();
    }
}

fn spawn_exit_confirm(commands: &mut Commands) {
    commands
        .spawn((
            ExitConfirmRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.04, 0.72)),
            GlobalZIndex(120),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(16.0),
                        padding: UiRect::all(Val::Px(28.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        min_width: Val::Px(320.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.1, 0.14)),
                    BorderColor(Color::srgb(0.55, 0.45, 0.3)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Return to title?"),
                        TextFont {
                            font_size: 26.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.92, 0.86)),
                    ));
                    panel.spawn((
                        Text::new("Progress is saved. Leave the homestead?"),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.68, 0.64)),
                    ));

                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(14.0),
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_button(
                                row,
                                ExitConfirmYes,
                                "Yes — Title",
                                Color::srgb(0.55, 0.22, 0.18),
                            );
                            spawn_button(
                                row,
                                ExitConfirmNo,
                                "No — Stay",
                                Color::srgb(0.22, 0.32, 0.28),
                            );
                        });

                    panel.spawn((
                        Text::new("Esc — cancel"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.52)),
                    ));
                });
        });
}

fn spawn_button<M: Component>(
    parent: &mut ChildBuilder<'_>,
    marker: M,
    label: &str,
    bg: Color,
) {
    parent
        .spawn((
            Button,
            marker,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg),
            BorderColor(Color::srgb(0.4, 0.35, 0.3)),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.93, 0.9)),
            ));
        });
}

/// Esc opens confirm, or cancels an open confirm. Does not steal Esc from fishing cancel.
pub fn handle_exit_confirm_escape(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<InventoryWindowOpen>,
    forge: Res<ForgeWindowOpen>,
    fishing: Option<Res<ActiveCast>>,
    mut open: ResMut<ExitConfirmOpen>,
    mut commands: Commands,
    roots: Query<Entity, With<ExitConfirmRoot>>,
    mut time: ResMut<Time<Virtual>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    // Inventory / forge / fishing claim Esc first.
    if inventory.0 || forge.0 {
        return;
    }
    if fishing.as_ref().is_some_and(|f| f.minigame_active()) {
        return;
    }

    if open.0 {
        close_exit_confirm(&mut commands, &mut open, &roots, &mut time);
        return;
    }

    open_exit_confirm(&mut commands, &mut open, &mut time);
}

pub fn handle_exit_confirm_buttons(
    mut commands: Commands,
    mut open: ResMut<ExitConfirmOpen>,
    roots: Query<Entity, With<ExitConfirmRoot>>,
    mut time: ResMut<Time<Virtual>>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    player: Query<&Transform, With<crate::overworld::movement::OverworldPlayer>>,
    mut profile: ResMut<crate::core::PlayerProfile>,
    mut dirty: ResMut<crate::core::ProfileDirty>,
    yes: Query<&Interaction, (Changed<Interaction>, With<ExitConfirmYes>)>,
    no: Query<&Interaction, (Changed<Interaction>, With<ExitConfirmNo>)>,
) {
    if !open.0 {
        return;
    }

    let yes_pressed = yes.iter().any(|i| *i == Interaction::Pressed);
    let no_pressed = no.iter().any(|i| *i == Interaction::Pressed);

    if yes_pressed {
        // Persist exact zone + position so resume returns here (e.g. lake pier).
        if let Ok(tf) = player.get_single() {
            if let Some(loc) =
                crate::player::SavedLocation::from_game_state(*state.get(), tf.translation.truncate())
            {
                profile.location = loc;
                dirty.mark();
            }
        }
        close_exit_confirm(&mut commands, &mut open, &roots, &mut time);
        next_state.set(GameState::Title);
        return;
    }

    if no_pressed {
        close_exit_confirm(&mut commands, &mut open, &roots, &mut time);
    }
}

pub fn cleanup_exit_confirm(
    mut commands: Commands,
    mut open: ResMut<ExitConfirmOpen>,
    roots: Query<Entity, With<ExitConfirmRoot>>,
    mut time: ResMut<Time<Virtual>>,
) {
    if open.0 || !roots.is_empty() {
        close_exit_confirm(&mut commands, &mut open, &roots, &mut time);
    }
}
