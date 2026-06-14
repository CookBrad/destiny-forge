use bevy::prelude::*;

#[derive(Component)]
pub struct ControlsHelp;

pub fn spawn_controls_help(mut commands: Commands) {
    commands.spawn((
        Text::new(controls_text_for_hub()),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        ControlsHelp,
    ));
}

pub fn update_controls_for_state(
    state: Res<State<crate::core::GameState>>,
    mut query: Query<&mut Text, With<ControlsHelp>>,
) {
    let Ok(mut text) = query.get_single_mut() else {
        return;
    };

    **text = match state.get() {
        crate::core::GameState::Hub => controls_text_for_hub(),
        crate::core::GameState::Dungeon => controls_text_for_dungeon(),
        crate::core::GameState::AssetLoading => "Loading...".to_string(),
    };
}

fn controls_text_for_hub() -> String {
    [
        "Hub (top-down)",
        "WASD — move",
        "E at dark door — enter dungeon",
        "Up/Down at forge — select recipe",
        "F at forge — craft",
    ]
    .join("\n")
}

fn controls_text_for_dungeon() -> String {
    [
        "Dungeon (side-scroller)",
        "A/D — move  |  Space — jump  |  J — attack",
        "E — carve corpse / exit at golden door",
    ]
    .join("\n")
}