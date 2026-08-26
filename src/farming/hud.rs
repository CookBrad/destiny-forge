//! Simple bottom-left tool readout (keys 1–4). Not the #22 drag hotbar.

use bevy::prelude::*;

use super::tools::{EquippedTool, HomesteadTool};

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
                padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.05, 0.04, 0.72)),
            BorderColor(Color::srgba(0.55, 0.42, 0.22, 0.9)),
        ))
        .with_children(|root| {
            root.spawn((
                ToolHudLabel,
                Text::new(hud_text(equipped.0)),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.86, 0.72)),
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
    let text = hud_text(equipped.0);
    for mut label in &mut labels {
        label.0 = text.clone();
    }
}

fn hud_text(selected: HomesteadTool) -> String {
    HomesteadTool::ALL
        .iter()
        .map(|tool| {
            if *tool == selected {
                format!("[{} {}]", tool.hotkey(), tool.label())
            } else {
                format!("{} {}", tool.hotkey(), tool.label())
            }
        })
        .collect::<Vec<_>>()
        .join("  ·  ")
}
