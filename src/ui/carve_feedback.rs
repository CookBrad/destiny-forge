use bevy::prelude::*;

use crate::dungeon::carve::{CarveState, LootLog};

const PROGRESS_WIDTH: f32 = 180.0;
const PROGRESS_HEIGHT: f32 = 14.0;
const LOG_MAX_ENTRIES: usize = 6;
const LOG_LIFETIME_SECS: f32 = 3.5;
const LOG_FADE_SECS: f32 = 0.75;

#[derive(Component)]
pub struct CarveProgressHud;

#[derive(Component)]
pub struct CarveProgressFill;

#[derive(Component)]
pub struct CarveProgressLabel;

#[derive(Component)]
pub struct LootLogHud;

#[derive(Component)]
pub struct LootLogLine {
    pub age: f32,
}

pub fn spawn_carve_feedback_ui(mut commands: Commands) {
    // Progress bar — bottom-center, above skill bar.
    commands
        .spawn((
            CarveProgressHud,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(100.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                CarveProgressLabel,
                Text::new("Carving…"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.88, 0.8)),
            ));
            root.spawn((
                Node {
                    width: Val::Px(PROGRESS_WIDTH),
                    height: Val::Px(PROGRESS_HEIGHT),
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.9)),
                BorderColor(Color::srgba(0.45, 0.4, 0.3, 0.95)),
            ))
            .with_children(|track| {
                track.spawn((
                    CarveProgressFill,
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.55, 0.82, 0.42)),
                ));
            });
        });

    // Loot log — lower-left, stacks upward.
    commands.spawn((
        LootLogHud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            bottom: Val::Px(100.0),
            flex_direction: FlexDirection::ColumnReverse,
            row_gap: Val::Px(3.0),
            width: Val::Px(280.0),
            ..default()
        },
    ));
}

pub fn cleanup_carve_feedback_ui(
    mut commands: Commands,
    progress: Query<Entity, With<CarveProgressHud>>,
    log: Query<Entity, With<LootLogHud>>,
    mut loot_log: ResMut<LootLog>,
) {
    loot_log.pending.clear();
    for entity in &progress {
        commands.entity(entity).try_despawn_recursive();
    }
    for entity in &log {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn sync_carve_progress_ui(
    carve_state: Res<CarveState>,
    mut hud: Query<&mut Visibility, With<CarveProgressHud>>,
    mut fill: Query<&mut Node, With<CarveProgressFill>>,
    mut label: Query<&mut Text, With<CarveProgressLabel>>,
) {
    let active = carve_state.target.is_some() && !carve_state.timer.finished();
    let progress = if active {
        carve_state.timer.fraction().clamp(0.0, 1.0)
    } else {
        0.0
    };

    for mut visibility in &mut hud {
        *visibility = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut node in &mut fill {
        node.width = Val::Percent(progress * 100.0);
    }

    if let Ok(mut text) = label.get_single_mut() {
        let pct = (progress * 100.0).round() as u32;
        let next = if active {
            format!("Carving… {pct}%")
        } else {
            "Carving…".to_string()
        };
        if text.as_str() != next {
            text.0 = next;
        }
    }
}

pub fn drain_loot_log_to_ui(
    mut commands: Commands,
    mut loot_log: ResMut<LootLog>,
    log_root: Query<Entity, With<LootLogHud>>,
    lines: Query<Entity, With<LootLogLine>>,
) {
    if loot_log.pending.is_empty() {
        return;
    }

    let Ok(root) = log_root.get_single() else {
        loot_log.pending.clear();
        return;
    };

    // Cap total lines.
    let existing = lines.iter().count();
    let overflow = existing
        .saturating_add(loot_log.pending.len())
        .saturating_sub(LOG_MAX_ENTRIES);
    if overflow > 0 {
        let mut oldest: Vec<_> = lines.iter().collect();
        // Despawn arbitrary extras; age system will clean more cleanly.
        for entity in oldest.into_iter().take(overflow) {
            commands.entity(entity).try_despawn_recursive();
        }
    }

    let entries: Vec<_> = loot_log.pending.drain(..).collect();
    for entry in entries {
        commands.entity(root).with_children(|parent| {
            parent.spawn((
                LootLogLine { age: 0.0 },
                Text::new(entry.text),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.92, 0.78)),
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.07, 0.1, 0.82)),
            ));
        });
    }
}

pub fn tick_loot_log_lines(
    time: Res<Time>,
    mut commands: Commands,
    mut lines: Query<(Entity, &mut LootLogLine, &mut TextColor, &mut BackgroundColor)>,
) {
    let dt = time.delta_secs();
    for (entity, mut line, mut text_color, mut bg) in &mut lines {
        line.age += dt;
        if line.age >= LOG_LIFETIME_SECS {
            commands.entity(entity).try_despawn_recursive();
            continue;
        }

        let fade_start = LOG_LIFETIME_SECS - LOG_FADE_SECS;
        let alpha = if line.age > fade_start {
            1.0 - ((line.age - fade_start) / LOG_FADE_SECS).clamp(0.0, 1.0)
        } else {
            1.0
        };

        text_color.0 = Color::srgba(0.95, 0.92, 0.78, alpha);
        bg.0 = Color::srgba(0.06, 0.07, 0.1, 0.82 * alpha);
    }
}
