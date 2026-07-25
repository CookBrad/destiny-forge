//! On-screen fishing timing bar: cursor, perfect/good zones, result feedback.

use bevy::prelude::*;

use super::cast::ActiveCast;
use super::logic::{
    CastPhase, DEFAULT_ZONE_CENTER, GOOD_ZONE_HALF, PERFECT_ZONE_HALF,
};

const BAR_WIDTH: f32 = 280.0;
const BAR_HEIGHT: f32 = 22.0;
const CURSOR_WIDTH: f32 = 4.0;

#[derive(Component)]
pub struct FishingBarRoot;

#[derive(Component)]
pub struct FishingBarCursor;

#[derive(Component)]
pub struct FishingBarLabel;

#[derive(Component)]
pub struct FishingGoodZone;

#[derive(Component)]
pub struct FishingPerfectZone;

/// Spawn the bar when a cast becomes visible; despawn when idle.
pub fn sync_fishing_bar_ui(
    mut commands: Commands,
    cast: Res<ActiveCast>,
    roots: Query<Entity, With<FishingBarRoot>>,
    mut cursor_q: Query<&mut Node, With<FishingBarCursor>>,
    mut label_q: Query<&mut Text, With<FishingBarLabel>>,
    mut good_q: Query<&mut Node, (With<FishingGoodZone>, Without<FishingBarCursor>, Without<FishingPerfectZone>)>,
    mut perfect_q: Query<&mut Node, (With<FishingPerfectZone>, Without<FishingBarCursor>, Without<FishingGoodZone>)>,
) {
    let visible = cast.bar_visible();
    let has_root = !roots.is_empty();

    if visible && !has_root {
        spawn_fishing_bar(&mut commands, &cast);
        return;
    }

    if !visible {
        for entity in &roots {
            commands.entity(entity).try_despawn_recursive();
        }
        return;
    }

    // Sync cursor + zones + label while active.
    let zone_center = cast.state.zone_center().unwrap_or(DEFAULT_ZONE_CENTER);
    let cursor = cast.state.cursor().unwrap_or(zone_center);

    for mut node in &mut good_q {
        let left = ((zone_center - GOOD_ZONE_HALF) * BAR_WIDTH).max(0.0);
        let width = (GOOD_ZONE_HALF * 2.0 * BAR_WIDTH).min(BAR_WIDTH - left);
        node.left = Val::Px(left);
        node.width = Val::Px(width);
    }
    for mut node in &mut perfect_q {
        let left = ((zone_center - PERFECT_ZONE_HALF) * BAR_WIDTH).max(0.0);
        let width = (PERFECT_ZONE_HALF * 2.0 * BAR_WIDTH).min(BAR_WIDTH - left);
        node.left = Val::Px(left);
        node.width = Val::Px(width);
    }
    for mut node in &mut cursor_q {
        // Hide cursor during result phase; show mid-wait.
        let show = matches!(cast.state.phase, CastPhase::Waiting { .. });
        if show {
            let left = (cursor * BAR_WIDTH - CURSOR_WIDTH * 0.5).clamp(0.0, BAR_WIDTH - CURSOR_WIDTH);
            node.left = Val::Px(left);
            node.width = Val::Px(CURSOR_WIDTH);
            node.display = Display::Flex;
        } else {
            node.display = Display::None;
        }
    }
    if let Some(label) = cast.state.result_label() {
        for mut text in &mut label_q {
            text.0 = label.to_string();
        }
    }
}

fn spawn_fishing_bar(commands: &mut Commands, cast: &ActiveCast) {
    let zone_center = cast.state.zone_center().unwrap_or(DEFAULT_ZONE_CENTER);
    let cursor = cast.state.cursor().unwrap_or(0.0);
    let label = cast
        .state
        .result_label()
        .unwrap_or("Space — Reel · Esc/Q — Cancel");

    let good_left = ((zone_center - GOOD_ZONE_HALF) * BAR_WIDTH).max(0.0);
    let good_width = (GOOD_ZONE_HALF * 2.0 * BAR_WIDTH).min(BAR_WIDTH - good_left);
    let perfect_left = ((zone_center - PERFECT_ZONE_HALF) * BAR_WIDTH).max(0.0);
    let perfect_width =
        (PERFECT_ZONE_HALF * 2.0 * BAR_WIDTH).min(BAR_WIDTH - perfect_left);
    let cursor_left =
        (cursor * BAR_WIDTH - CURSOR_WIDTH * 0.5).clamp(0.0, BAR_WIDTH - CURSOR_WIDTH);

    commands
        .spawn((
            FishingBarRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(118.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            GlobalZIndex(60),
        ))
        .with_children(|root| {
            root.spawn((
                FishingBarLabel,
                Text::new(label),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.88, 0.92, 0.95)),
            ));

            // Track
            root.spawn((
                Node {
                    width: Val::Px(BAR_WIDTH),
                    height: Val::Px(BAR_HEIGHT),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.1, 0.12, 0.16)),
                BorderColor(Color::srgb(0.45, 0.55, 0.65)),
            ))
            .with_children(|track| {
                // Good (yellow) zone
                track.spawn((
                    FishingGoodZone,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(good_left),
                        width: Val::Px(good_width),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.85, 0.72, 0.2, 0.55)),
                ));
                // Perfect (green) zone
                track.spawn((
                    FishingPerfectZone,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(perfect_left),
                        width: Val::Px(perfect_width),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.25, 0.85, 0.4, 0.75)),
                ));
                // Cursor
                track.spawn((
                    FishingBarCursor,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(cursor_left),
                        width: Val::Px(CURSOR_WIDTH),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.98, 0.98, 1.0)),
                ));
            });

            root.spawn((
                Text::new("Yellow = good · Green = perfect"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.55, 0.6, 0.65)),
            ));
        });
}

pub fn cleanup_fishing_bar(mut commands: Commands, roots: Query<Entity, With<FishingBarRoot>>) {
    for entity in &roots {
        commands.entity(entity).try_despawn_recursive();
    }
}
