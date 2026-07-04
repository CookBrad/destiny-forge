use bevy::prelude::*;

use crate::combat::{
    animate_special_weapon, animate_weapon_swing, apply_enemy_contact_damage,
    cleanup_special_weapon, deflect_projectiles_with_swing, despawn_block_weapon,
    enemy_shoot_projectiles, move_enemy_projectiles, resolve_deflected_projectile_hits,
    resolve_enemy_projectiles, resolve_special_move_hits, resolve_weapon_hits,
    start_player_attack, start_player_special_moves, sync_block_weapon, sync_sheathed_weapon,
    tick_hit_flash, tick_player_attack, tick_player_hit_flash, tick_player_special_moves,
    update_player_block,
};
use crate::core::{DungeonPlayState, GameState};
use crate::graphics::{follow_camera, init_dungeon_camera};

use super::animation::animate_player;
use super::boss::{resolve_boss_hazards, tick_boss_attacks};
use super::enemy::{move_enemies, track_boss_defeat};
use super::interaction::{ladder_interaction, update_ladder_prompt};
use super::movement::dungeon_movement;
use super::setup::{cleanup_dungeon, setup_dungeon};

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Dungeon),
            (setup_dungeon, init_dungeon_camera).chain(),
        )
            .add_systems(OnExit(GameState::Dungeon), cleanup_dungeon)
            .add_systems(
                Update,
                (
                    (
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
                        move_enemies,
                        resolve_boss_hazards,
                        enemy_shoot_projectiles,
                        move_enemy_projectiles,
                        deflect_projectiles_with_swing,
                        resolve_deflected_projectile_hits,
                        resolve_enemy_projectiles,
                        tick_player_hit_flash,
                        track_boss_defeat,
                        update_ladder_prompt,
                        ladder_interaction,
                    ),
                )
                    .chain()
                    .run_if(in_state(GameState::Dungeon))
                    .run_if(in_state(DungeonPlayState::Running)),
            );
    }
}