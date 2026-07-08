//! Homestead day / phase cue (top-right).

use bevy::prelude::*;

use crate::core::DayClock;

#[derive(Component)]
pub struct DayHudRoot;

#[derive(Component)]
pub struct DayHudLabel;

pub fn spawn_day_hud(commands: &mut Commands, clock: &DayClock) {
    commands
        .spawn((
            DayHudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(16.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.05, 0.04, 0.72)),
            BorderColor(Color::srgba(0.55, 0.45, 0.28, 0.85)),
        ))
        .with_children(|root| {
            root.spawn((
                DayHudLabel,
                Text::new(clock.hud_label()),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.94, 0.9, 0.8)),
            ));
        });
}

pub fn setup_day_hud(mut commands: Commands, clock: Res<DayClock>) {
    spawn_day_hud(&mut commands, &clock);
}

pub fn cleanup_day_hud(mut commands: Commands, roots: Query<Entity, With<DayHudRoot>>) {
    for entity in &roots {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn sync_day_hud(
    clock: Res<DayClock>,
    mut labels: Query<&mut Text, With<DayHudLabel>>,
) {
    if !clock.is_changed() {
        return;
    }
    let label = clock.hud_label();
    for mut text in &mut labels {
        text.0 = label.clone();
    }
}
