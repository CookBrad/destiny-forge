use bevy::prelude::*;

use crate::core::GameState;
use crate::player::Loadout;

const HUB_BUFF_CHIP_TOP_PX: f32 = 104.0;
const DUNGEON_BUFF_CHIP_TOP_PX: f32 = 12.0;

#[derive(Component)]
pub struct FoodBuffHudRoot;

#[derive(Component)]
pub struct FoodBuffHudLabel;

pub fn setup_food_buff_hud(
    mut commands: Commands,
    loadout: Res<Loadout>,
    game: Res<State<GameState>>,
) {
    let hud_top = match game.get() {
        GameState::Dungeon => DUNGEON_BUFF_CHIP_TOP_PX,
        _ => HUB_BUFF_CHIP_TOP_PX,
    };

    commands
        .spawn((
            FoodBuffHudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(hud_top),
                right: Val::Px(16.0),
                display: food_buff_hud_display(&loadout),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.05, 0.04, 0.72)),
            BorderColor(Color::srgba(0.85, 0.55, 0.22, 0.9)),
        ))
        .with_children(|root| {
            root.spawn((
                FoodBuffHudLabel,
                Text::new(food_buff_hud_text(&loadout)),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.98, 0.78, 0.45)),
            ));
        });
}

pub fn cleanup_food_buff_hud(mut commands: Commands, roots: Query<Entity, With<FoodBuffHudRoot>>) {
    for entity in &roots {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn sync_food_buff_hud(
    loadout: Res<Loadout>,
    mut roots: Query<&mut Node, With<FoodBuffHudRoot>>,
    mut labels: Query<&mut Text, With<FoodBuffHudLabel>>,
) {
    if !loadout.is_changed() {
        return;
    }

    for mut node in &mut roots {
        node.display = food_buff_hud_display(&loadout);
    }
    let hud_text = food_buff_hud_text(&loadout);
    for mut text in &mut labels {
        text.0 = hud_text.clone();
    }
}

fn food_buff_hud_display(loadout: &Loadout) -> Display {
    if loadout.food_buff.is_some() {
        Display::Flex
    } else {
        Display::None
    }
}

fn food_buff_hud_text(loadout: &Loadout) -> String {
    match loadout.food_buff {
        Some(food) => format!(
            "Fed: {}  {}",
            food.display_name(),
            loadout.active_food_buff().bonus_summary()
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
        assert_eq!(food_buff_hud_display(&loadout), Display::None);

        loadout.food_buff = Some(MaterialId::PotatoStew);
        assert_eq!(food_buff_hud_display(&loadout), Display::Flex);
        assert_eq!(
            food_buff_hud_text(&loadout),
            "Fed: Potato Stew  +2 defense, +10% carve"
        );
    }
}
