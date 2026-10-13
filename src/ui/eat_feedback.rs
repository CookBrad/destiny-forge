use bevy::prelude::*;

use crate::items::MaterialId;
use crate::player::Loadout;

const EAT_POPUP_LIFETIME_SECS: f32 = 1.6;
const EAT_POPUP_POP_SECS: f32 = 0.18;
const EAT_POPUP_FADE_SECS: f32 = 0.5;
const EAT_POPUP_START_FONT_SIZE: f32 = 14.0;
const EAT_POPUP_PEAK_FONT_SIZE: f32 = 24.0;
const EAT_POPUP_SETTLED_FONT_SIZE: f32 = 20.0;
const EAT_POPUP_TOP_PERCENT: f32 = 38.0;
const EAT_POPUP_RISE_PX: f32 = 28.0;
const EAT_POPUP_Z_INDEX: i32 = 150;
const EAT_POPUP_TEXT_RGB: (f32, f32, f32) = (0.98, 0.82, 0.48);
const EAT_POPUP_BG_RGBA: (f32, f32, f32, f32) = (0.12, 0.07, 0.03, 0.88);

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoodEaten {
    pub food: MaterialId,
}

#[derive(Component)]
pub struct EatFeedbackPopup {
    pub age_secs: f32,
}

#[derive(Component)]
pub struct EatFeedbackPopupText;

pub fn spawn_eat_feedback_popup(
    mut commands: Commands,
    mut food_eaten: EventReader<FoodEaten>,
    loadout: Res<Loadout>,
    existing_popups: Query<Entity, With<EatFeedbackPopup>>,
) {
    let Some(latest_meal) = food_eaten.read().last().copied() else {
        return;
    };
    for entity in &existing_popups {
        commands.entity(entity).try_despawn_recursive();
    }

    let popup_text = eat_popup_text(latest_meal.food, &loadout);
    commands
        .spawn((
            EatFeedbackPopup { age_secs: 0.0 },
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                top: Val::Percent(EAT_POPUP_TOP_PERCENT),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(EAT_POPUP_Z_INDEX),
        ))
        .with_children(|popup| {
            popup.spawn((
                EatFeedbackPopupText,
                Text::new(popup_text),
                TextFont {
                    font_size: EAT_POPUP_START_FONT_SIZE,
                    ..default()
                },
                TextColor(eat_popup_text_color(1.0)),
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(eat_popup_background_color(1.0)),
            ));
        });
}

pub fn animate_eat_feedback_popup(
    real_time: Res<Time<Real>>,
    mut commands: Commands,
    mut popups: Query<(Entity, &mut EatFeedbackPopup, &mut Node, &Children)>,
    mut popup_texts: Query<
        (&mut TextFont, &mut TextColor, &mut BackgroundColor),
        With<EatFeedbackPopupText>,
    >,
) {
    let delta_secs = real_time.delta_secs();
    for (entity, mut popup, mut node, children) in &mut popups {
        popup.age_secs += delta_secs;
        if popup.age_secs >= EAT_POPUP_LIFETIME_SECS {
            commands.entity(entity).try_despawn_recursive();
            continue;
        }

        node.margin.top = Val::Px(-eat_popup_rise_px(popup.age_secs));
        let opacity = eat_popup_opacity(popup.age_secs);
        for child in children.iter() {
            if let Ok((mut font, mut text_color, mut background)) = popup_texts.get_mut(*child) {
                font.font_size = eat_popup_font_size(popup.age_secs);
                text_color.0 = eat_popup_text_color(opacity);
                background.0 = eat_popup_background_color(opacity);
            }
        }
    }
}

pub fn cleanup_eat_feedback_popup(
    mut commands: Commands,
    popups: Query<Entity, With<EatFeedbackPopup>>,
) {
    for entity in &popups {
        commands.entity(entity).try_despawn_recursive();
    }
}

fn eat_popup_text(food: MaterialId, loadout: &Loadout) -> String {
    format!(
        "Ate {}!  {}",
        food.display_name(),
        loadout.active_food_buff().bonus_summary()
    )
}

fn eat_popup_font_size(age_secs: f32) -> f32 {
    let half_pop_secs = EAT_POPUP_POP_SECS / 2.0;
    if age_secs < half_pop_secs {
        let progress = age_secs / half_pop_secs;
        EAT_POPUP_START_FONT_SIZE
            + (EAT_POPUP_PEAK_FONT_SIZE - EAT_POPUP_START_FONT_SIZE) * progress
    } else if age_secs < EAT_POPUP_POP_SECS {
        let progress = (age_secs - half_pop_secs) / half_pop_secs;
        EAT_POPUP_PEAK_FONT_SIZE
            - (EAT_POPUP_PEAK_FONT_SIZE - EAT_POPUP_SETTLED_FONT_SIZE) * progress
    } else {
        EAT_POPUP_SETTLED_FONT_SIZE
    }
}

fn eat_popup_rise_px(age_secs: f32) -> f32 {
    EAT_POPUP_RISE_PX * (age_secs / EAT_POPUP_LIFETIME_SECS).clamp(0.0, 1.0)
}

fn eat_popup_opacity(age_secs: f32) -> f32 {
    let fade_start_secs = EAT_POPUP_LIFETIME_SECS - EAT_POPUP_FADE_SECS;
    if age_secs <= fade_start_secs {
        1.0
    } else {
        1.0 - ((age_secs - fade_start_secs) / EAT_POPUP_FADE_SECS).clamp(0.0, 1.0)
    }
}

fn eat_popup_text_color(opacity: f32) -> Color {
    let (red, green, blue) = EAT_POPUP_TEXT_RGB;
    Color::srgba(red, green, blue, opacity)
}

fn eat_popup_background_color(opacity: f32) -> Color {
    let (red, green, blue, alpha) = EAT_POPUP_BG_RGBA;
    Color::srgba(red, green, blue, alpha * opacity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn popup_names_food_and_active_bonus() {
        let mut loadout = Loadout::default();
        loadout.food_buff = Some(MaterialId::PotatoStew);
        assert_eq!(
            eat_popup_text(MaterialId::PotatoStew, &loadout),
            "Ate Potato Stew!  +2 defense, +10% carve"
        );
    }

    #[test]
    fn popup_pops_then_settles_and_fades_out() {
        assert_eq!(eat_popup_font_size(0.0), EAT_POPUP_START_FONT_SIZE);
        assert!(
            (eat_popup_font_size(EAT_POPUP_POP_SECS / 2.0) - EAT_POPUP_PEAK_FONT_SIZE).abs() < 1e-4
        );
        assert_eq!(eat_popup_font_size(1.0), EAT_POPUP_SETTLED_FONT_SIZE);
        assert_eq!(eat_popup_opacity(0.5), 1.0);
        assert!(eat_popup_opacity(EAT_POPUP_LIFETIME_SECS - 0.1) < 0.5);
        assert_eq!(eat_popup_opacity(EAT_POPUP_LIFETIME_SECS), 0.0);
        assert!(eat_popup_rise_px(1.0) > eat_popup_rise_px(0.2));
    }

    #[test]
    fn eating_spawns_one_popup() {
        let mut app = App::new();
        app.add_event::<FoodEaten>().init_resource::<Loadout>();
        app.world_mut().send_event(FoodEaten {
            food: MaterialId::RoastTurnip,
        });

        app.world_mut()
            .run_system_once(spawn_eat_feedback_popup)
            .unwrap();
        app.world_mut().flush();

        let popup_count = app
            .world_mut()
            .query::<&EatFeedbackPopup>()
            .iter(app.world())
            .count();
        assert_eq!(popup_count, 1);
    }
}
