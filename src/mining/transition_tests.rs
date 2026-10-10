//! Headless `App` proof of the real homestead <-> mine transition path.
//!
//! Runs the production systems `overworld_interaction` and `mine_interaction` with the
//! same `in_state` / `inventory_closed` / `forge_closed` run conditions the plugins use,
//! plus the real `cleanup_overworld` / `cleanup_mine` OnExit hooks, against the real
//! `OverworldLayout::homestead()` MineEntrance zone and a `MineExit` ladder at `EXIT_TILE`.
//!
//! Only the asset-loading OnEnter setups (`setup_overworld`, `setup_mine`) are stood in
//! for: they call `OverworldArt::load`, which needs `AssetServer` + render `Image` assets
//! that a windowless `MinimalPlugins` app does not have. The stand-ins spawn the same
//! gameplay components at the same positions production uses (`ENTRY_TILE`,
//! `mine_return_tile()`), and insert the same fresh `MapTransitionCooldown`.

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
use super::layout::{MineEntity, MineExit, ENTRY_TILE, EXIT_TILE};
use super::setup::cleanup_mine;

fn player_at(pixels: Vec2) -> Transform {
    world_transform(
        Vec2::new(pixels.x, center_on_surface(pixels.y, PLAYER_SPRITE_HEIGHT)),
        5.0,
    )
}

/// Stand-in for `setup_overworld` minus art: player at the `OverworldEntry` spawn.
fn enter_overworld_headless(mut commands: Commands, entry: Option<Res<OverworldEntry>>) {
    let spawn = entry.map(|e| *e).unwrap_or_default();
    let start = match spawn {
        OverworldEntry::MineReturn => mine_return_tile(),
        // Test starts at the mine mouth: centre of the real MineEntrance zone.
        _ => mine_entrance_rect().center(),
    };
    commands.spawn((player_at(start), OverworldPlayer, OverworldVelocity::default()));
    commands.insert_resource(MapTransitionCooldown::default());
    commands.insert_resource(OverworldLayout::homestead());
    commands.remove_resource::<OverworldEntry>();
}

/// Stand-in for `setup_mine` minus art: ladder at `EXIT_TILE`, player at `ENTRY_TILE`.
fn enter_mine_headless(mut commands: Commands) {
    commands.spawn((
        world_transform(tile_center(EXIT_TILE.0, EXIT_TILE.1), 1.8),
        MineExit,
        MineEntity,
    ));
    commands.spawn((
        player_at(tile_center(ENTRY_TILE.0, ENTRY_TILE.1)),
        OverworldPlayer,
        OverworldVelocity::default(),
    ));
    commands.insert_resource(MapTransitionCooldown::default());
}

fn headless_app() -> App {
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
        // Same OnEnter ordering as the plugins, art loading stubbed (see module docs).
        .add_systems(OnEnter(GameState::Overworld), enter_overworld_headless)
        .add_systems(OnExit(GameState::Overworld), cleanup_overworld)
        .add_systems(OnEnter(GameState::Mine), enter_mine_headless)
        .add_systems(OnExit(GameState::Mine), cleanup_mine)
        // Real interaction systems with the plugins' run conditions.
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

fn state(app: &App) -> GameState {
    app.world().resource::<State<GameState>>().get().clone()
}

fn go_to(app: &mut App, target: GameState) {
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(target.clone());
    app.update();
    assert_eq!(state(app), target);
}

/// One frame with KeyE just pressed (InputPlugin is absent, so press/clear by hand).
fn press_e(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyE);
    app.update();
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(KeyCode::KeyE);
    keys.clear();
}

fn pending(app: &App) -> Option<GameState> {
    match app.world().resource::<NextState<GameState>>() {
        NextState::Pending(s) => Some(s.clone()),
        NextState::Unchanged => None,
    }
}

fn player_position(app: &mut App) -> Vec2 {
    let mut q = app
        .world_mut()
        .query_filtered::<&Transform, With<OverworldPlayer>>();
    let players: Vec<Vec2> = q
        .iter(app.world())
        .map(|t| t.translation.truncate())
        .collect();
    assert_eq!(players.len(), 1, "exactly one player after the transition");
    players[0]
}

fn expire_cooldown(app: &mut App) {
    app.world_mut()
        .resource_mut::<MapTransitionCooldown>()
        .0
        .tick(Duration::from_secs(1));
}

#[test]
fn key_e_at_mine_entrance_enters_mine_and_ladder_returns_to_homestead() {
    let mut app = headless_app();
    app.update();
    go_to(&mut app, GameState::Overworld);

    // 1. Overworld: player stands in the real MineEntrance zone.
    let start = player_position(&mut app);
    let layout = app.world().resource::<OverworldLayout>().clone();
    assert_eq!(
        layout.zone_at(start).map(|z| z.zone),
        Some(HomesteadZone::MineEntrance)
    );

    // 2. KeyE -> overworld_interaction queues GameState::Mine; next frame applies it.
    press_e(&mut app);
    assert_eq!(pending(&app), Some(GameState::Mine));
    app.update();
    assert_eq!(state(&app), GameState::Mine);
    assert!(app.world().get_resource::<OverworldLayout>().is_none(), "cleanup_overworld ran");

    // 3. Mine: E at the ladder is ignored until the transition cooldown expires.
    let ladder = tile_center(EXIT_TILE.0, EXIT_TILE.1);
    {
        let mut q = app
            .world_mut()
            .query_filtered::<&mut Transform, With<OverworldPlayer>>();
        let mut player = q.single_mut(app.world_mut());
        *player = player_at(ladder);
    }
    press_e(&mut app);
    assert_eq!(pending(&app), None, "fresh cooldown blocks an instant bounce");

    // KeyE at the ladder -> mine_interaction sets MineReturn + GameState::Overworld.
    expire_cooldown(&mut app);
    press_e(&mut app);
    assert_eq!(pending(&app), Some(GameState::Overworld));
    assert!(matches!(
        app.world().get_resource::<OverworldEntry>(),
        Some(OverworldEntry::MineReturn)
    ));
    app.update();
    assert_eq!(state(&app), GameState::Overworld);

    // Back on the homestead at the mine-return tile, mine entities torn down.
    let mut mine_q = app.world_mut().query_filtered::<Entity, With<MineEntity>>();
    assert_eq!(mine_q.iter(app.world()).count(), 0, "cleanup_mine ran");
    let back = player_position(&mut app);
    assert_eq!(back, player_at(mine_return_tile()).translation.truncate());
    assert_ne!(
        app.world()
            .resource::<OverworldLayout>()
            .zone_at(back)
            .map(|z| z.zone),
        Some(HomesteadZone::MineEntrance),
        "return tile must not sit inside the entrance zone"
    );
}

#[test]
fn key_e_away_from_mine_entrance_stays_on_homestead() {
    let mut app = headless_app();
    app.update();
    go_to(&mut app, GameState::Overworld);
    {
        let mut q = app
            .world_mut()
            .query_filtered::<&mut Transform, With<OverworldPlayer>>();
        let mut player = q.single_mut(app.world_mut());
        *player = player_at(mine_return_tile() + Vec2::new(-400.0, 0.0));
    }
    press_e(&mut app);
    assert_eq!(pending(&app), None);
    app.update();
    assert_eq!(state(&app), GameState::Overworld);
}
