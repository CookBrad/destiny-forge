use bevy::prelude::*;

use crate::combat::SkillBindings;
use crate::core::{DungeonPlayState, DungeonUiTeardown, GameState};
use crate::dungeon::move_enemies;
use crate::forging::RecipeBook;
use crate::graphics::reset_camera_zoom;

use super::carve_feedback::{
    cleanup_carve_feedback_ui, drain_loot_log_to_ui, spawn_carve_feedback_ui,
    sync_carve_progress_ui, tick_loot_log_lines,
};
use super::day_hud::{cleanup_day_hud, setup_day_hud, sync_day_hud};
use super::energy_hud::{cleanup_energy_hud, setup_energy_hud, sync_energy_hud};
use super::hotbar::{
    cleanup_hotbar, handle_hotbar_slot_clicks, select_hotbar_slot_input, setup_hotbar,
    sync_hotbar_ui,
};
use super::health_bars::{
    cleanup_health_bars, despawn_orphan_enemy_health_bars, setup_health_bar_assets,
    spawn_enemy_health_bars, spawn_player_health_bar, update_enemy_health_bars,
    update_player_health_bar, HealthBarAssets,
};
use super::forge_window::{
    cleanup_forge_window, forge_window_open, handle_forge_close_input, handle_forge_craft_input,
    handle_forge_recipe_cycle, sync_forge_display, ForgeSelectedRecipe, ForgeWindowOpen,
};
use super::interaction_prompt::{
    cleanup_interaction_prompt, setup_interaction_prompt, sync_interaction_prompt_ui,
    InteractionPrompt,
};
use super::inventory_window::{
    cleanup_inventory_window, handle_inventory_close_button, handle_inventory_slot_click,
    inventory_window_open, sync_inventory_display, sync_inventory_hover_tooltip,
    toggle_inventory_window, InventorySelectedSlot, InventoryWindowOpen,
};
use super::menu::{
    cleanup_death_menu, cleanup_pause_menu, cleanup_title_menu, death_menu_input,
    ensure_time_running, open_pause_menu, pause_game_time, pause_menu_input, resume_game_time,
    set_title_clear_color, spawn_death_menu, spawn_pause_menu, spawn_title_menu, sync_title_hint,
};
use super::title_profiles::{
    handle_profile_rename_input, handle_title_profile_card_clicks,
    handle_title_profile_keyboard_shortcuts, handle_title_profile_rename_clicks,
    sync_title_profile_cards, ProfileRenameState,
};
use super::pause_audio::{handle_pause_audio_input, sync_pause_audio_display};
use super::profile_picker::{refresh_profile_picker, ProfilePicker};
use super::skill_bar::{
    cleanup_skill_bar, handle_skill_bar_drag, setup_skill_icon_assets, spawn_skill_bar,
    sync_skill_bar, update_skill_bar_drag_ghost, SkillBarDrag,
};

fn clear_profile_rename_state(mut rename: ResMut<ProfileRenameState>) {
    rename.active = None;
}

fn reset_title_camera(mut camera: Query<&mut Projection, With<Camera2d>>) {
    for mut projection in &mut camera {
        reset_camera_zoom(&mut projection);
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProfilePicker::default())
            .init_resource::<ProfileRenameState>()
            .init_resource::<InventoryWindowOpen>()
            .init_resource::<InventorySelectedSlot>()
            .init_resource::<ForgeWindowOpen>()
            .init_resource::<ForgeSelectedRecipe>()
            .init_resource::<InteractionPrompt>()
            .init_resource::<RecipeBook>()
            .init_resource::<HealthBarAssets>()
            .init_resource::<SkillBindings>()
            .init_resource::<SkillBarDrag>()
            .add_systems(Startup, (setup_health_bar_assets, setup_skill_icon_assets))
            .add_systems(
                OnEnter(GameState::Title),
                (
                    ensure_time_running,
                    reset_title_camera,
                    set_title_clear_color,
                    refresh_profile_picker,
                    spawn_title_menu,
                    cleanup_day_hud,
                    cleanup_energy_hud,
                    cleanup_hotbar,
                    cleanup_interaction_prompt,
                )
                    .chain(),
            )
            .add_systems(
                OnExit(GameState::Title),
                (cleanup_title_menu, clear_profile_rename_state),
            )
            .add_systems(
                OnEnter(GameState::Overworld),
                (
                    setup_day_hud,
                    setup_energy_hud,
                    setup_hotbar,
                    setup_interaction_prompt,
                ),
            )
            .add_systems(
                OnEnter(GameState::Forest),
                (setup_day_hud, setup_energy_hud),
            )
            .add_systems(OnEnter(GameState::Dungeon), setup_interaction_prompt)
            .add_systems(
                OnExit(GameState::Overworld),
                (
                    cleanup_day_hud,
                    cleanup_energy_hud,
                    cleanup_hotbar,
                    cleanup_interaction_prompt,
                ),
            )
            .add_systems(
                OnExit(GameState::Forest),
                (cleanup_day_hud, cleanup_energy_hud),
            )
            .add_systems(
                Update,
                (
                    sync_day_hud.run_if(
                        in_state(GameState::Overworld).or(in_state(GameState::Forest)),
                    ),
                    sync_energy_hud.run_if(
                        in_state(GameState::Overworld).or(in_state(GameState::Forest)),
                    ),
                    (
                        select_hotbar_slot_input,
                        handle_hotbar_slot_clicks,
                        sync_hotbar_ui,
                    )
                        .run_if(in_state(GameState::Overworld)),
                    sync_interaction_prompt_ui.run_if(
                        in_state(GameState::Overworld).or(in_state(GameState::Dungeon)),
                    ),
                ),
            )
            .add_systems(
                Update,
                (
                    handle_title_profile_card_clicks,
                    handle_title_profile_rename_clicks,
                    handle_title_profile_keyboard_shortcuts,
                    handle_profile_rename_input,
                    sync_title_profile_cards,
                    sync_title_hint,
                )
                    .run_if(in_state(GameState::Title)),
            )
            .add_systems(
                OnEnter(DungeonPlayState::Paused),
                (pause_game_time, spawn_pause_menu),
            )
            .add_systems(
                OnExit(DungeonPlayState::Paused),
                (resume_game_time, cleanup_pause_menu),
            )
            .add_systems(
                Update,
                (
                    toggle_inventory_window,
                    (
                        handle_inventory_close_button,
                        handle_inventory_slot_click,
                        sync_inventory_display,
                        sync_inventory_hover_tooltip,
                    )
                        .chain()
                        .run_if(inventory_window_open),
                )
                    .run_if(
                        in_state(GameState::Overworld)
                            .or(in_state(GameState::Forest))
                            .or(in_state(DungeonPlayState::Running))
                            .or(in_state(DungeonPlayState::Paused)),
                    ),
            )
            .add_systems(
                Update,
                (
                    open_pause_menu.run_if(in_state(DungeonPlayState::Running)),
                    pause_menu_input.run_if(in_state(DungeonPlayState::Paused)),
                    (
                        handle_pause_audio_input,
                        sync_pause_audio_display,
                    )
                        .chain()
                        .run_if(in_state(DungeonPlayState::Paused)),
                    death_menu_input.run_if(in_state(DungeonPlayState::Dead)),
                )
                    .run_if(in_state(GameState::Dungeon)),
            )
            .add_systems(
                OnEnter(GameState::Dungeon),
                (
                    setup_health_bar_assets,
                    spawn_skill_bar,
                    spawn_player_health_bar,
                    spawn_enemy_health_bars,
                    spawn_carve_feedback_ui,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(DungeonPlayState::Dead),
                (pause_game_time, spawn_death_menu),
            )
            .add_systems(
                OnExit(DungeonPlayState::Dead),
                (resume_game_time, cleanup_death_menu),
            )
            .add_systems(
                Update,
                (
                    handle_forge_close_input,
                    handle_forge_recipe_cycle,
                    handle_forge_craft_input,
                    sync_forge_display,
                )
                    .chain()
                    .run_if(in_state(GameState::Overworld))
                    .run_if(forge_window_open),
            )
            .add_systems(
                OnExit(GameState::Overworld),
                (cleanup_inventory_window, cleanup_forge_window),
            )
            .add_systems(OnExit(GameState::Forest), cleanup_inventory_window)
            .add_systems(
                OnExit(GameState::Dungeon),
                (
                    ensure_time_running,
                    cleanup_inventory_window,
                    cleanup_pause_menu,
                    cleanup_death_menu,
                    cleanup_skill_bar,
                    cleanup_health_bars,
                    cleanup_interaction_prompt,
                    cleanup_carve_feedback_ui,
                )
                    .chain()
                    .in_set(DungeonUiTeardown),
            )
            .add_systems(
                Update,
                (
                    handle_skill_bar_drag,
                    update_skill_bar_drag_ghost.after(handle_skill_bar_drag),
                    sync_skill_bar.after(update_skill_bar_drag_ghost),
                    update_player_health_bar,
                    despawn_orphan_enemy_health_bars,
                    spawn_enemy_health_bars,
                    update_enemy_health_bars.after(move_enemies),
                    sync_carve_progress_ui,
                    drain_loot_log_to_ui,
                    tick_loot_log_lines,
                )
                    .run_if(in_state(GameState::Dungeon))
                    .run_if(
                        in_state(DungeonPlayState::Running)
                            .or(in_state(DungeonPlayState::Paused)),
                    ),
            );
    }
}
