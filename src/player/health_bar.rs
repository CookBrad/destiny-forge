use crate::enemy::Health;
use bevy::prelude::*;
use bevy::ui::BackgroundColor;

#[derive(Component)]
pub struct PlayerHealthBar;

pub fn setup_player_health_bar(mut commands: Commands) {
    // Spawn health bar fill (the actual health indicator)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(20.0),
            height: Val::Px(200.0), // Will be updated based on health
            right: Val::Px(20.0),
            bottom: Val::Px(20.0), // Anchor at bottom
            ..default()
        },
        BackgroundColor(Color::srgb(0.0, 0.6, 0.0)), // Dark green health bar (will change based on health)
        PlayerHealthBar,
        Name::new("PlayerHealthBarFill"),
    ));
}

pub fn update_player_health_bar(
    time: Res<Time>,
    player_query: Query<&Health, With<crate::player::Player>>,
    mut health_bar_query: Query<(&mut Node, &mut BackgroundColor, &Name), With<PlayerHealthBar>>,
) {
    let Ok(player_health) = player_query.get_single() else {
        return;
    };

    let health_percentage = player_health.current / player_health.max;
    let base_height = 200.0;
    let current_height = base_height * health_percentage;

    for (mut node, mut bg_color, name) in health_bar_query.iter_mut() {
        if name.as_str() == "PlayerHealthBarFill" {
            node.height = Val::Px(current_height.max(0.0)); // Ensure non-negative

            // Shake effect when health is in red zone (below 30%)
            let shake_amount = if health_percentage <= 0.3 {
                // Shake more intensely as health gets lower
                let intensity = 1.0 - (health_percentage / 0.3); // 0.0 to 1.0
                (time.elapsed_secs() * 20.0).sin() * intensity * 3.0 // Shake 3px max
            } else {
                0.0
            };

            node.right = Val::Px(20.0 + shake_amount);
            node.bottom = Val::Px(20.0); // Always keep bottom fixed

            // Change color based on health: dark green -> yellow -> red
            // Smooth color transitions
            if health_percentage > 0.6 {
                // Dark green (full health)
                *bg_color = Color::srgb(0.0, 0.6, 0.0).into();
            } else if health_percentage > 0.3 {
                // Transition from green to yellow
                let t = (health_percentage - 0.3) / 0.3; // 0.0 to 1.0 between 30% and 60%
                let r = t; // Red increases
                let g = 0.6 + (0.4 * t); // Green increases from 0.6 to 1.0
                *bg_color = Color::srgb(r, g, 0.0).into();
            } else {
                // Transition from yellow to red
                let t = health_percentage / 0.3; // 0.0 to 1.0 between 0% and 30%
                let r = 1.0; // Red stays at max
                let g = t; // Green decreases from 1.0 to 0.0
                *bg_color = Color::srgb(r, g, 0.0).into();
            }
        }
    }
}
