//! Active food buff chip (homestead, forest, dungeon).

use bevy::prelude::*;

use crate::core::GameState;
use crate::player::Loadout;

const HUB_TOP: f32 = 104.0;
const DUNGEON_TOP: f32 = 12.0;

#[derive(Component)]
pub struct BuffHudRoot;

#[derive(Component)]
pub struct BuffHudLabel;

pub fn setup_buff_hud(mut commands: Commands, loadout: Res<Loadout>, game: Res<State<GameState>>) {
    let top = match game.get() {
        GameState::Dungeon => DUNGEON_TOP,
        _ => HUB_TOP,
    };

    commands
        .spawn((
            BuffHudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(top),
                right: Val::Px(16.0),
                display: hud_display(&loadout),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.05, 0.04, 0.72)),
            BorderColor(Color::srgba(0.85, 0.55, 0.22, 0.9)),
        ))
        .with_children(|root| {
            root.spawn((
                BuffHudLabel,
                Text::new(buff_label(&loadout)),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.98, 0.78, 0.45)),
            ));
        });
}

pub fn cleanup_buff_hud(mut commands: Commands, roots: Query<Entity, With<BuffHudRoot>>) {
    for entity in &roots {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn sync_buff_hud(
    loadout: Res<Loadout>,
    mut roots: Query<&mut Node, With<BuffHudRoot>>,
    mut labels: Query<&mut Text, With<BuffHudLabel>>,
) {
    if !loadout.is_changed() {
        return;
    }

    for mut node in &mut roots {
        node.display = hud_display(&loadout);
    }
    let label = buff_label(&loadout);
    for mut text in &mut labels {
        text.0 = label.clone();
    }
}

fn hud_display(loadout: &Loadout) -> Display {
    if loadout.food_buff.is_some() {
        Display::Flex
    } else {
        Display::None
    }
}

fn buff_label(loadout: &Loadout) -> String {
    match loadout.food_buff {
        Some(food) => format!(
            "Fed: {}  {}",
            food.display_name(),
            loadout.active_food_buff().summary()
        ),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::MaterialId;

    #[test]
    fn label_names_food_and_bonus() {
        let mut loadout = Loadout::default();
        assert_eq!(hud_display(&loadout), Display::None);

        loadout.food_buff = Some(MaterialId::PotatoStew);
        assert_eq!(hud_display(&loadout), Display::Flex);
        assert_eq!(
            buff_label(&loadout),
            "Fed: Potato Stew  +2 defense, +10% carve"
        );
    }
}
