//! Equipped homestead tool cue (bottom-left on overworld).

use bevy::prelude::*;

use crate::farming::{EquippedTool, HomesteadTool};

#[derive(Component)]
pub struct ToolHudRoot;

#[derive(Component)]
pub struct ToolHudLabel;

pub fn setup_tool_hud(mut commands: Commands, equipped: Res<EquippedTool>) {
    commands
        .spawn((
            ToolHudRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.05, 0.04, 0.72)),
            BorderColor(Color::srgba(0.45, 0.55, 0.35, 0.9)),
        ))
        .with_children(|root| {
            root.spawn((
                ToolHudLabel,
                Text::new(tool_label(equipped.0)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.92, 0.82)),
            ));
            root.spawn((
                Text::new("1–4 tools · Space/LMB use"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.55, 0.55, 0.5)),
            ));
        });
}

pub fn cleanup_tool_hud(mut commands: Commands, roots: Query<Entity, With<ToolHudRoot>>) {
    for entity in &roots {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn sync_tool_hud(
    equipped: Res<EquippedTool>,
    mut labels: Query<&mut Text, With<ToolHudLabel>>,
) {
    if !equipped.is_changed() {
        return;
    }
    let label = tool_label(equipped.0);
    for mut text in &mut labels {
        text.0 = label.clone();
    }
}

fn tool_label(tool: HomesteadTool) -> String {
    let idx = tool.hotkey_index().map(|i| i + 1).unwrap_or(0);
    format!("Tool [{idx}] {}", tool.label())
}
