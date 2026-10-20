use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::{
    perform_sleep, CalendarDate, CalendarRules, DayClock, GameState, PlayerProfile, ProfileDirty,
    Season, SeasonRollover, ToolEnergy,
};
use crate::farming::{advance_all_plots_on_sleep, capture_plots, CropPlot};
use crate::forging::RecipeBook;
use crate::graphics::INTERACT_DISTANCE;
use crate::input::{Action, ActionState};
use crate::mining::respawn_ore_nodes_on_sleep;
use crate::player::Loadout;
use crate::ui::forge_window::{
    open_forge_window, ForgeCraftSources, ForgeSelectedRecipe, ForgeWindowOpen,
};
use crate::ui::interaction_prompt::{best_prompt, InteractionPrompt, PromptKind};
use crate::ui::inventory_window::InventoryWindowOpen;

use super::layout::{homestead_forest_transition, Bed, HomesteadZone, OverworldLayout};
use super::movement::{MapTransitionCooldown, OverworldPlayer, OverworldVelocity};

const ZONE_INTERACT_RANGE: f32 = INTERACT_DISTANCE * 2.0;
const BED_INTERACT_RANGE: f32 = INTERACT_DISTANCE * 1.5;

#[derive(SystemParam)]
pub(crate) struct OverworldForgeOpen<'w, 's> {
    commands: Commands<'w, 's>,
    game_inventory: Res<'w, crate::items::Inventory>,
    forge_windows: Query<'w, 's, Entity, With<crate::ui::forge_window::ForgeWindow>>,
    time: ResMut<'w, Time<Virtual>>,
    forge: ResMut<'w, ForgeWindowOpen>,
    forge_recipe: ResMut<'w, ForgeSelectedRecipe>,
    loadout: Res<'w, Loadout>,
    recipes: Res<'w, RecipeBook>,
}

pub fn overworld_interaction(
    actions: Res<ActionState>,
    inventory: Res<InventoryWindowOpen>,
    layout: Res<OverworldLayout>,
    cooldown: Res<MapTransitionCooldown>,
    player: Query<(&Transform, &OverworldVelocity), With<OverworldPlayer>>,
    mut next_state: ResMut<NextState<GameState>>,
    forge_open: OverworldForgeOpen,
) {
    let OverworldForgeOpen {
        mut commands,
        game_inventory,
        forge_windows,
        mut time,
        mut forge,
        mut forge_recipe,
        loadout,
        recipes,
    } = forge_open;
    let Ok((transform, velocity)) = player.get_single() else {
        return;
    };

    if inventory.0 || forge.0 {
        return;
    }

    let position = transform.translation.truncate();

    if let Some(zone) = layout.zone_at(position) {
        let near = distance_to_zone(position, &zone.bounds) <= ZONE_INTERACT_RANGE;
        match zone.zone {
            HomesteadZone::Forge if near && actions.just_pressed(Action::Interact) => {
                open_forge_window(
                    &mut forge,
                    &mut forge_recipe,
                    &mut commands,
                    ForgeCraftSources {
                        inventory: &game_inventory,
                        loadout: &loadout,
                        recipes: &recipes,
                    },
                    &forge_windows,
                    &mut time,
                );
            }
            HomesteadZone::DungeonGate if near && actions.just_pressed(Action::Interact) => {
                // Day cost is applied OnEnter(Dungeon) so all entry paths share one hook.
                next_state.set(GameState::Dungeon);
            }
            HomesteadZone::MineEntrance if near && actions.just_pressed(Action::Interact) => {
                next_state.set(GameState::Mine);
            }
            _ => {}
        }
    }

    if cooldown.0.finished() && homestead_forest_transition().contains(position) && velocity.y > 1.0
    {
        next_state.set(GameState::Forest);
    }

    if actions.just_pressed(Action::Cancel) {
        next_state.set(GameState::Title);
    }
}

#[derive(SystemParam)]
pub(crate) struct SleepAtBed<'w, 's> {
    day_clock: ResMut<'w, DayClock>,
    tool_energy: ResMut<'w, ToolEnergy>,
    season: ResMut<'w, Season>,
    rules: Res<'w, CalendarRules>,
    season_rollovers: EventWriter<'w, SeasonRollover>,
    profile: ResMut<'w, PlayerProfile>,
    profile_dirty: ResMut<'w, ProfileDirty>,
    clear: ResMut<'w, ClearColor>,
    plots: Query<'w, 's, &'static mut CropPlot>,
    loadout: ResMut<'w, Loadout>,
}

pub fn try_sleep_at_bed(
    actions: Res<ActionState>,
    inventory: Res<InventoryWindowOpen>,
    forge: Res<ForgeWindowOpen>,
    player: Query<&Transform, With<OverworldPlayer>>,
    beds: Query<&Transform, With<Bed>>,
    sleep: SleepAtBed,
) {
    let SleepAtBed {
        mut day_clock,
        mut tool_energy,
        mut season,
        rules,
        mut season_rollovers,
        mut profile,
        mut profile_dirty,
        mut clear,
        mut plots,
        mut loadout,
    } = sleep;
    if inventory.0 || forge.0 || !actions.just_pressed(Action::Interact) {
        return;
    }

    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let position = player_transform.translation.truncate();
    if !near_any_bed(position, &beds) {
        return;
    }

    let outcome = perform_sleep(&mut day_clock, &mut tool_energy, &mut season, &rules);
    if let Some(rollover) = outcome.season_rollover {
        season_rollovers.send(rollover);
    }
    let day = outcome.calendar_day;
    advance_all_plots_on_sleep(&mut plots);
    loadout.clear_food_buff();
    profile.calendar_day = day_clock.calendar_day;
    profile.day_phase = day_clock.phase;
    profile.tool_energy = tool_energy.current;
    profile.crop_plots = capture_plots(plots.iter());
    respawn_ore_nodes_on_sleep(&mut profile);
    profile_dirty.mark();
    clear.0 = day_clock.phase.ambient_clear_color();
    let date = CalendarDate::from_calendar_day(day, &rules);
    info!(
        "Slept — calendar day {day}, {} {} year {} ({}). Tool energy restored; crops advanced; mine ore respawned.",
        date.season.display_name(),
        date.day_of_season,
        date.year,
        date.weekday.display_name(),
    );
}

/// Show tooltip for forge / dungeon gate / bed when in range.
pub fn update_overworld_interaction_prompt(
    inventory: Res<InventoryWindowOpen>,
    forge: Res<ForgeWindowOpen>,
    layout: Res<OverworldLayout>,
    player: Query<&Transform, With<OverworldPlayer>>,
    beds: Query<&Transform, With<Bed>>,
    mut prompt: ResMut<InteractionPrompt>,
) {
    if inventory.0 || forge.0 {
        prompt.clear();
        return;
    }

    let Ok(player_transform) = player.get_single() else {
        prompt.clear();
        return;
    };
    let position = player_transform.translation.truncate();
    let kind = best_prompt(&overworld_prompt_candidates(position, &layout, &beds));
    prompt.set(kind);
}

fn overworld_prompt_candidates(
    position: Vec2,
    layout: &OverworldLayout,
    beds: &Query<&Transform, With<Bed>>,
) -> Vec<PromptKind> {
    let mut candidates = Vec::with_capacity(3);

    if near_any_bed(position, beds) {
        candidates.push(PromptKind::Sleep);
    }

    if let Some(zone) = layout.zone_at(position) {
        if distance_to_zone(position, &zone.bounds) <= ZONE_INTERACT_RANGE {
            match zone.zone {
                HomesteadZone::Forge => candidates.push(PromptKind::OpenForge),
                HomesteadZone::DungeonGate => candidates.push(PromptKind::EnterDungeon),
                HomesteadZone::MineEntrance => candidates.push(PromptKind::EnterMine),
                _ => {}
            }
        }
    }

    candidates
}

fn near_any_bed(position: Vec2, beds: &Query<&Transform, With<Bed>>) -> bool {
    beds.iter()
        .any(|bed| position.distance(bed.translation.truncate()) <= BED_INTERACT_RANGE)
}

fn distance_to_zone(position: Vec2, bounds: &Rect) -> f32 {
    position.distance(bounds.center())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    use crate::items::MaterialId;

    #[test]
    fn sleeping_ends_the_food_buff() {
        let mut app = App::new();
        let loadout = Loadout {
            food_buff: Some(MaterialId::RoastTurnip),
            ..Loadout::default()
        };
        app.insert_resource(crate::input::action_state_after_pressing(
            &[KeyCode::KeyE],
            &[],
        ))
        .insert_resource(loadout)
        .init_resource::<InventoryWindowOpen>()
        .init_resource::<ForgeWindowOpen>()
        .init_resource::<DayClock>()
        .init_resource::<ToolEnergy>()
        .init_resource::<Season>()
        .init_resource::<CalendarRules>()
        .add_event::<SeasonRollover>()
        .init_resource::<PlayerProfile>()
        .init_resource::<ProfileDirty>()
        .init_resource::<ClearColor>();
        app.world_mut()
            .spawn((OverworldPlayer, Transform::default()));
        app.world_mut().spawn((Bed, Transform::default()));

        app.world_mut().run_system_once(try_sleep_at_bed).unwrap();

        assert_eq!(app.world().resource::<Loadout>().food_buff, None);
        assert_eq!(app.world().resource::<DayClock>().calendar_day, 2);
        assert_eq!(*app.world().resource::<Season>(), Season::Spring);
        assert!(app
            .world_mut()
            .resource_mut::<Events<SeasonRollover>>()
            .drain()
            .next()
            .is_none());
    }

    #[test]
    fn sleeping_out_of_winter_emits_a_season_rollover() {
        let mut app = App::new();
        app.insert_resource(crate::input::action_state_after_pressing(
            &[KeyCode::KeyE],
            &[],
        ))
        .insert_resource(Loadout::default())
        .init_resource::<InventoryWindowOpen>()
        .init_resource::<ForgeWindowOpen>()
        .init_resource::<DayClock>()
        .init_resource::<ToolEnergy>()
        .init_resource::<Season>()
        .init_resource::<CalendarRules>()
        .add_event::<SeasonRollover>()
        .init_resource::<PlayerProfile>()
        .init_resource::<ProfileDirty>()
        .init_resource::<ClearColor>();
        let rules = *app.world().resource::<CalendarRules>();
        let last_day = rules.season_length() * 4;
        app.world_mut().resource_mut::<DayClock>().calendar_day = last_day;
        *app.world_mut().resource_mut::<Season>() =
            CalendarDate::from_calendar_day(last_day, &rules).season;
        app.world_mut()
            .spawn((OverworldPlayer, Transform::default()));
        app.world_mut().spawn((Bed, Transform::default()));

        app.world_mut().run_system_once(try_sleep_at_bed).unwrap();

        let rollover = app
            .world_mut()
            .resource_mut::<Events<SeasonRollover>>()
            .drain()
            .next()
            .expect("season rollover");
        assert_eq!(rollover.ended, Season::Winter);
        assert_eq!(rollover.started, Season::Spring);
        assert_eq!(rollover.year, 2);
        assert_eq!(rollover.day_of_season, 1);
        assert_eq!(
            rollover.weekday,
            CalendarDate::from_calendar_day(last_day + 1, &rules).weekday
        );
        assert_eq!(rollover.calendar_day, last_day + 1);
        assert_eq!(*app.world().resource::<Season>(), Season::Spring);
    }
}
