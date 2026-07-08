use bevy::prelude::*;

use crate::combat::{
    animate_player_death, animate_special_weapon, animate_weapon_swing, apply_enemy_contact_damage,
    cleanup_special_weapon, deflect_projectiles_with_swing, despawn_block_weapon, detect_player_death,
    enemy_shoot_projectiles, finish_player_death, hide_death_weapons, move_enemy_projectiles,
    resolve_deflected_projectile_hits, resolve_enemy_projectiles, resolve_special_move_hits,
    resolve_weapon_hits, start_player_attack, start_player_special_moves, sync_block_weapon,
    sync_sheathed_weapon, tick_hit_flash, tick_hit_stop, tick_player_attack, tick_player_death,
    tick_player_hit_flash, tick_player_special_moves, tick_special_cooldowns, update_player_block,
    HitStop, SpecialCooldownState,
};
use crate::core::{DungeonPlayState, DungeonUiTeardown, GameState};
use crate::ui::inventory_window::inventory_closed;
use crate::graphics::{follow_camera, init_dungeon_camera};

use super::animation::animate_player;
use super::boss::{resolve_boss_hazards, tick_boss_attacks, tick_boss_phase_flash};
use super::carve::carve_corpses;
use super::enemy::track_boss_defeat;
use super::enemy_movement::move_enemies;
use super::interaction::{ladder_interaction, update_dungeon_interaction_prompt};
use super::movement::dungeon_movement;
use super::setup::{cleanup_dungeon, retry_dungeon, setup_dungeon};

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<super::carve::CarveState>()
            .init_resource::<HitStop>()
            .init_resource::<SpecialCooldownState>()
            .add_systems(
            OnEnter(GameState::Dungeon),
            (
                setup_dungeon,
                init_dungeon_camera,
                |mut next_play: ResMut<NextState<DungeonPlayState>>| {
                    next_play.set(DungeonPlayState::Running);
                },
            )
                .chain(),
        )
            .add_systems(
                OnExit(GameState::Dungeon),
                cleanup_dungeon.after(DungeonUiTeardown),
            )
            .add_systems(
                Update,
                detect_player_death
                    .run_if(in_state(GameState::Dungeon))
                    .run_if(in_state(DungeonPlayState::Running)),
            )
            .add_systems(
                Update,
                (
                    tick_player_death,
                    hide_death_weapons,
                    animate_player_death,
                    follow_camera,
                    finish_player_death,
                )
                    .chain()
                    .run_if(in_state(GameState::Dungeon))
                    .run_if(in_state(DungeonPlayState::Dying)),
            )
            .add_systems(
                Update,
                retry_dungeon
                    .run_if(in_state(GameState::Dungeon))
                    .run_if(in_state(DungeonPlayState::Dead)),
            )
            .add_systems(
                Update,
                (
                    (
                        tick_hit_stop,
                        tick_special_cooldowns,
                        update_player_block,
                        start_player_attack,
                        start_player_special_moves,
                        tick_player_attack,
                        tick_player_special_moves,
                        animate_weapon_swing,
                        cleanup_special_weapon,
                        sync_block_weapon,
                        despawn_block_weapon,
                        sync_sheathed_weapon,
                        dungeon_movement,
                        animate_special_weapon,
                        follow_camera,
                        animate_player,
                    ),
                    (
                        resolve_weapon_hits,
                        resolve_special_move_hits,
                        apply_enemy_contact_damage,
                        tick_hit_flash,
                        tick_boss_attacks,
                        tick_boss_phase_flash,
                        move_enemies,
                        resolve_boss_hazards,
                        enemy_shoot_projectiles,
                        move_enemy_projectiles,
                        deflect_projectiles_with_swing,
                        resolve_deflected_projectile_hits,
                        resolve_enemy_projectiles,
                        tick_player_hit_flash,
                        track_boss_defeat,
                        carve_corpses,
                        ladder_interaction,
                        update_dungeon_interaction_prompt,
                    ),
                )
                    .chain()
                    .run_if(in_state(GameState::Dungeon))
                    .run_if(in_state(DungeonPlayState::Running))
                    .run_if(inventory_closed),
            );
    }
}