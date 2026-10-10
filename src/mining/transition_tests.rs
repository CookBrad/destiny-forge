use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::core::GameState;
use crate::forging::RecipeBook;
use crate::graphics::{center_on_surface, world_transform};
use crate::items::Inventory;
use crate::overworld::interaction::overworld_interaction;
use crate::overworld::layout::{
    mine_entrance_rect, mine_return_tile, tile_center, HomesteadZone, OverworldLayout,
};
use crate::overworld::movement::{MapTransitionCooldown, OverworldPlayer, OverworldVelocity};
use crate::overworld::setup::{cleanup_overworld, OverworldEntry};
use crate::overworld::sprites::PLAYER_SPRITE_HEIGHT;
use crate::player::Loadout;
use crate::ui::forge_window::{forge_closed, ForgeSelectedRecipe, ForgeWindowOpen};
use crate::ui::inventory_window::{inventory_closed, InventoryWindowOpen};

use super::interaction::mine_interaction;
use super::layout::{MineEntity, MineLadder, ARRIVAL_TILE, LADDER_TILE};
use super::setup::cleanup_mine;

fn player_transform_standing_at(pixels: Vec2) -> Transform {
    world_transform(
        Vec2::new(pixels.x, center_on_surface(pixels.y, PLAYER_SPRITE_HEIGHT)),
        5.0,
    )
}

fn spawn_homestead_player_without_art(mut commands: Commands, entry: Option<Res<OverworldEntry>>) {
    let entry = entry.map(|entry| *entry).unwrap_or_default();
    let start = match entry {
        OverworldEntry::MineReturn => mine_return_tile(),
        _ => mine_entrance_rect().center(),
    };
    commands.spawn((
        player_transform_standing_at(start),
        OverworldPlayer,
        OverworldVelocity::default(),
    ));
    commands.insert_resource(MapTransitionCooldown::default());
    commands.insert_resource(OverworldLayout::homestead());
    commands.remove_resource::<OverworldEntry>();
}

fn spawn_mine_ladder_and_player_without_art(mut commands: Commands) {
    commands.spawn((
        world_transform(tile_center(LADDER_TILE.0, LADDER_TILE.1), 1.8),
        MineLadder,
        MineEntity,
    ));
    commands.spawn((
        player_transform_standing_at(tile_center(ARRIVAL_TILE.0, ARRIVAL_TILE.1)),
        OverworldPlayer,
        OverworldVelocity::default(),
    ));
    commands.insert_resource(MapTransitionCooldown::default());
}

fn headless_app_with_real_map_transition_systems() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<GameState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<InventoryWindowOpen>()
        .init_resource::<ForgeWindowOpen>()
        .init_resource::<ForgeSelectedRecipe>()
        .init_resource::<Inventory>()
        .init_resource::<Loadout>()
        .init_resource::<RecipeBook>()
        .add_systems(
            OnEnter(GameState::Overworld),
            spawn_homestead_player_without_art,
        )
        .add_systems(OnExit(GameState::Overworld), cleanup_overworld)
        .add_systems(
            OnEnter(GameState::Mine),
            spawn_mine_ladder_and_player_without_art,
        )
        .add_systems(OnExit(GameState::Mine), cleanup_mine)
        .add_systems(
            Update,
            overworld_interaction
                .run_if(in_state(GameState::Overworld))
                .run_if(inventory_closed)
                .run_if(forge_closed),
        )
        .add_systems(
            Update,
            mine_interaction
                .run_if(in_state(GameState::Mine))
                .run_if(inventory_closed),
        );
    app
}

fn current_state(app: &App) -> GameState {
    app.world().resource::<State<GameState>>().get().clone()
}

fn transition_to(app: &mut App, target: GameState) {
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(target.clone());
    app.update();
    assert_eq!(current_state(app), target);
}

fn press_key_e_for_one_frame(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyE);
    app.update();
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(KeyCode::KeyE);
    keys.clear();
}

fn queued_state(app: &App) -> Option<GameState> {
    match app.world().resource::<NextState<GameState>>() {
        NextState::Pending(queued) => Some(queued.clone()),
        NextState::Unchanged => None,
    }
}

fn only_player_position(app: &mut App) -> Vec2 {
    let mut player_query = app
        .world_mut()
        .query_filtered::<&Transform, With<OverworldPlayer>>();
    let players: Vec<Vec2> = player_query
        .iter(app.world())
        .map(|transform| transform.translation.truncate())
        .collect();
    assert_eq!(players.len(), 1, "exactly one player after the transition");
    players[0]
}

fn expire_map_transition_cooldown(app: &mut App) {
    app.world_mut()
        .resource_mut::<MapTransitionCooldown>()
        .0
        .tick(Duration::from_secs(1));
}

fn move_player_to(app: &mut App, pixels: Vec2) {
    let mut player_query = app
        .world_mut()
        .query_filtered::<&mut Transform, With<OverworldPlayer>>();
    let mut player = player_query.single_mut(app.world_mut());
    *player = player_transform_standing_at(pixels);
}

#[test]
fn key_e_at_mine_entrance_enters_mine_and_ladder_returns_to_homestead() {
    let mut app = headless_app_with_real_map_transition_systems();
    app.update();
    transition_to(&mut app, GameState::Overworld);

    let start_position = only_player_position(&mut app);
    let layout = app.world().resource::<OverworldLayout>().clone();
    assert_eq!(
        layout.zone_at(start_position).map(|zone| zone.zone),
        Some(HomesteadZone::MineEntrance)
    );

    press_key_e_for_one_frame(&mut app);
    assert_eq!(queued_state(&app), Some(GameState::Mine));
    app.update();
    assert_eq!(current_state(&app), GameState::Mine);
    assert!(
        app.world().get_resource::<OverworldLayout>().is_none(),
        "cleanup_overworld ran"
    );

    let ladder_position = tile_center(LADDER_TILE.0, LADDER_TILE.1);
    move_player_to(&mut app, ladder_position);
    press_key_e_for_one_frame(&mut app);
    assert_eq!(
        queued_state(&app),
        None,
        "fresh cooldown blocks an instant bounce"
    );

    expire_map_transition_cooldown(&mut app);
    press_key_e_for_one_frame(&mut app);
    assert_eq!(queued_state(&app), Some(GameState::Overworld));
    assert!(matches!(
        app.world().get_resource::<OverworldEntry>(),
        Some(OverworldEntry::MineReturn)
    ));
    app.update();
    assert_eq!(current_state(&app), GameState::Overworld);

    let mut mine_entities = app.world_mut().query_filtered::<Entity, With<MineEntity>>();
    assert_eq!(
        mine_entities.iter(app.world()).count(),
        0,
        "cleanup_mine ran"
    );
    let returned_position = only_player_position(&mut app);
    assert_eq!(
        returned_position,
        player_transform_standing_at(mine_return_tile())
            .translation
            .truncate()
    );
    assert_ne!(
        app.world()
            .resource::<OverworldLayout>()
            .zone_at(returned_position)
            .map(|zone| zone.zone),
        Some(HomesteadZone::MineEntrance),
        "return tile must not sit inside the entrance zone"
    );
}

#[test]
fn key_e_away_from_mine_entrance_stays_on_homestead() {
    let mut app = headless_app_with_real_map_transition_systems();
    app.update();
    transition_to(&mut app, GameState::Overworld);
    move_player_to(&mut app, mine_return_tile() + Vec2::new(-400.0, 0.0));
    press_key_e_for_one_frame(&mut app);
    assert_eq!(queued_state(&app), None);
    app.update();
    assert_eq!(current_state(&app), GameState::Overworld);
}
