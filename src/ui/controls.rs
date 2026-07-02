use bevy::prelude::*;

use crate::dungeon::LadderPrompt;

#[derive(Component)]
pub struct DungeonHud;

pub fn spawn_controls_help(mut commands: Commands) {
    commands.spawn((
        Text::new(controls_text(false)),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.92, 0.92)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        DungeonHud,
    ));
}

pub fn update_controls_help(
    prompt: Res<LadderPrompt>,
    mut text: Query<&mut Text, With<DungeonHud>>,
) {
    let Ok(mut text) = text.get_single_mut() else {
        return;
    };

    if text.as_str() != controls_text(prompt.visible) {
        **text = controls_text(prompt.visible);
    }
}

pub fn cleanup_controls_help(
    mut commands: Commands,
    hud: Query<Entity, With<DungeonHud>>,
) {
    for entity in &hud {
        commands.entity(entity).despawn_recursive();
    }
}

fn controls_text(near_ladder: bool) -> String {
    let mut lines = vec![
        "Dungeon Floor 1".to_string(),
        "A/D — move   Space — jump   1 — attack".to_string(),
    ];

    if near_ladder {
        lines.push("E — exit to hub (stub)".to_string());
    } else {
        lines.push("Reach the ladder to return".to_string());
    }

    lines.join("\n")
}