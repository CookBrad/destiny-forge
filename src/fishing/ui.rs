//! Vertical Stardew-style fishing UI: track, green bar, fish marker, progress.

use bevy::prelude::*;

use super::cast::ActiveCast;
use super::logic::{CastPhase, FightSim};

const TRACK_W: f32 = 36.0;
const TRACK_H: f32 = 220.0;
const FISH_SIZE: f32 = 14.0;

#[derive(Component)]
pub struct FishingBarRoot;

#[derive(Component)]
pub struct FishingGreenBar;

#[derive(Component)]
pub struct FishingFishMarker;

#[derive(Component)]
pub struct FishingProgressFill;

#[derive(Component)]
pub struct FishingBarLabel;

#[derive(Component)]
pub struct FishingHintLabel;

pub fn sync_fishing_bar_ui(
    mut commands: Commands,
    cast: Res<ActiveCast>,
    roots: Query<Entity, With<FishingBarRoot>>,
    mut green_q: Query<&mut Node, With<FishingGreenBar>>,
    mut fish_q: Query<&mut Node, (With<FishingFishMarker>, Without<FishingGreenBar>)>,
    mut prog_q: Query<&mut Node, (With<FishingProgressFill>, Without<FishingGreenBar>, Without<FishingFishMarker>)>,
    mut label_q: Query<&mut Text, (With<FishingBarLabel>, Without<FishingHintLabel>)>,
    mut hint_q: Query<&mut Text, (With<FishingHintLabel>, Without<FishingBarLabel>)>,
) {
    let show_full = cast.bar_visible();
    let show_phase = cast.minigame_active();
    let has_root = !roots.is_empty();

    if show_phase && !has_root {
        spawn_fishing_ui(&mut commands, &cast);
        return;
    }
    if !show_phase {
        for entity in &roots {
            commands.entity(entity).try_despawn_recursive();
        }
        return;
    }

    // Phase label always.
    if let Some(label) = cast.state.result_label() {
        for mut text in &mut label_q {
            text.0 = label.to_string();
        }
    }

    // Full vertical contest only during fight / result with fight data.
    if let Some(sim) = cast.state.fight() {
        sync_fight_widgets(sim, &mut green_q, &mut fish_q, &mut prog_q);
        for mut text in &mut hint_q {
            text.0 = format!("Catch meter {:.0}%", sim.progress * 100.0);
        }
    } else if matches!(cast.state.phase, CastPhase::ShowingResult { .. }) {
        for mut text in &mut hint_q {
            text.0 = " ".to_string();
        }
    } else {
        // Casting / waiting: hide green bar widgets via zero size
        for mut node in &mut green_q {
            node.height = Val::Px(0.0);
        }
        for mut text in &mut hint_q {
            text.0 = match cast.state.phase {
                CastPhase::Casting { .. } => "Line out…".to_string(),
                CastPhase::WaitingBite { .. } => "…nibble…".to_string(),
                _ => String::new(),
            };
        }
    }

    let _ = show_full;
}

fn sync_fight_widgets(
    sim: &FightSim,
    green_q: &mut Query<&mut Node, With<FishingGreenBar>>,
    fish_q: &mut Query<&mut Node, (With<FishingFishMarker>, Without<FishingGreenBar>)>,
    prog_q: &mut Query<
        &mut Node,
        (
            With<FishingProgressFill>,
            Without<FishingGreenBar>,
            Without<FishingFishMarker>,
        ),
    >,
) {
    // UI Y grows downward; game axis has 0 at bottom → invert for `bottom` positioning.
    for mut node in green_q.iter_mut() {
        let bottom_px = sim.bar_bottom * TRACK_H;
        let height_px = sim.bar_height * TRACK_H;
        node.bottom = Val::Px(bottom_px);
        node.height = Val::Px(height_px.max(4.0));
        node.width = Val::Px(TRACK_W - 4.0);
        node.display = Display::Flex;
    }
    for mut node in fish_q.iter_mut() {
        let bottom_px = sim.fish_y * TRACK_H - FISH_SIZE * 0.5;
        node.bottom = Val::Px(bottom_px.clamp(0.0, TRACK_H - FISH_SIZE));
        node.display = Display::Flex;
    }
    for mut node in prog_q.iter_mut() {
        node.height = Val::Px((sim.progress * TRACK_H).clamp(0.0, TRACK_H));
    }
}

fn spawn_fishing_ui(commands: &mut Commands, cast: &ActiveCast) {
    let label = cast.state.result_label().unwrap_or("Fishing");
    let sim = cast.state.fight().cloned().unwrap_or_default();

    commands
        .spawn((
            FishingBarRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(100.0),
                right: Val::Px(28.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.08, 0.12, 0.88)),
            BorderColor(Color::srgb(0.35, 0.55, 0.7)),
            GlobalZIndex(70),
        ))
        .with_children(|root| {
            root.spawn((
                FishingBarLabel,
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.94, 0.98)),
            ));

            // Horizontal row: progress | track
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::FlexEnd,
                height: Val::Px(TRACK_H),
                ..default()
            })
            .with_children(|row| {
                // Progress meter (fills from bottom)
                row.spawn((
                    Node {
                        width: Val::Px(10.0),
                        height: Val::Px(TRACK_H),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::FlexEnd,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.14, 0.16)),
                    BorderColor(Color::srgb(0.3, 0.35, 0.4)),
                ))
                .with_children(|track| {
                    track.spawn((
                        FishingProgressFill,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(sim.progress * TRACK_H),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.95, 0.75, 0.2)),
                    ));
                });

                // Main vertical track
                row.spawn((
                    Node {
                        width: Val::Px(TRACK_W),
                        height: Val::Px(TRACK_H),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.28, 0.4)),
                    BorderColor(Color::srgb(0.4, 0.6, 0.75)),
                ))
                .with_children(|track| {
                    let bottom_px = sim.bar_bottom * TRACK_H;
                    let height_px = sim.bar_height * TRACK_H;
                    track.spawn((
                        FishingGreenBar,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(2.0),
                            bottom: Val::Px(bottom_px),
                            width: Val::Px(TRACK_W - 4.0),
                            height: Val::Px(height_px.max(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.85, 0.35, 0.85)),
                    ));

                    let fish_bottom = (sim.fish_y * TRACK_H - FISH_SIZE * 0.5)
                        .clamp(0.0, TRACK_H - FISH_SIZE);
                    track.spawn((
                        FishingFishMarker,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px((TRACK_W - FISH_SIZE) * 0.5),
                            bottom: Val::Px(fish_bottom),
                            width: Val::Px(FISH_SIZE),
                            height: Val::Px(FISH_SIZE),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.95, 0.55, 0.2)),
                        BorderColor(Color::srgb(1.0, 0.9, 0.7)),
                    ));
                });
            });

            root.spawn((
                FishingHintLabel,
                Text::new("Hold Space to raise the green bar"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.72, 0.78)),
            ));

            root.spawn((
                Text::new("Esc / Q — cancel"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.55, 0.6)),
            ));
        });
}

pub fn cleanup_fishing_bar(mut commands: Commands, roots: Query<Entity, With<FishingBarRoot>>) {
    for entity in &roots {
        commands.entity(entity).try_despawn_recursive();
    }
}
