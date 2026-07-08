//! Homestead tool-energy meter (overworld / forest).

use bevy::prelude::*;

use crate::core::ToolEnergy;

const BAR_WIDTH: f32 = 120.0;
const BAR_HEIGHT: f32 = 10.0;

#[derive(Component)]
pub struct EnergyHudRoot;

#[derive(Component)]
pub struct EnergyHudFill;

#[derive(Component)]
pub struct EnergyHudLabel;

pub fn setup_energy_hud(mut commands: Commands, energy: Res<ToolEnergy>) {
    spawn_energy_hud(&mut commands, &energy);
}

pub fn spawn_energy_hud(commands: &mut Commands, energy: &ToolEnergy) {
    commands
        .spawn((
            EnergyHudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(52.0),
                right: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.05, 0.04, 0.72)),
            BorderColor(Color::srgba(0.35, 0.55, 0.4, 0.9)),
        ))
        .with_children(|root| {
            root.spawn((
                EnergyHudLabel,
                Text::new(energy_label(energy)),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.9, 0.78)),
            ));

            root.spawn((
                Node {
                    width: Val::Px(BAR_WIDTH),
                    height: Val::Px(BAR_HEIGHT),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.14, 0.12)),
                BorderColor(Color::srgb(0.25, 0.32, 0.26)),
            ))
            .with_children(|track| {
                track.spawn((
                    EnergyHudFill,
                    Node {
                        width: Val::Px(BAR_WIDTH * energy.fraction()),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(fill_color(energy.fraction())),
                ));
            });
        });
}

pub fn cleanup_energy_hud(mut commands: Commands, roots: Query<Entity, With<EnergyHudRoot>>) {
    for entity in &roots {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn sync_energy_hud(
    energy: Res<ToolEnergy>,
    mut labels: Query<&mut Text, With<EnergyHudLabel>>,
    mut fills: Query<(&mut Node, &mut BackgroundColor), With<EnergyHudFill>>,
) {
    if !energy.is_changed() {
        return;
    }

    let fraction = energy.fraction();
    let label = energy_label(&energy);
    let color = fill_color(fraction);

    for mut text in &mut labels {
        text.0 = label.clone();
    }
    for (mut node, mut bg) in &mut fills {
        node.width = Val::Px(BAR_WIDTH * fraction);
        bg.0 = color;
    }
}

fn energy_label(energy: &ToolEnergy) -> String {
    format!("Energy  {:.0}/{:.0}", energy.current, energy.max)
}

fn fill_color(fraction: f32) -> Color {
    if fraction <= 0.2 {
        Color::srgb(0.72, 0.28, 0.22)
    } else if fraction <= 0.5 {
        Color::srgb(0.78, 0.62, 0.28)
    } else {
        Color::srgb(0.32, 0.68, 0.42)
    }
}
