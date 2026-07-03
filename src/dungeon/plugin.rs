use bevy::prelude::*;

use crate::combat::{
    animate_weapon_swing, apply_enemy_contact_damage, deflect_projectiles_with_swing,
    despawn_block_weapon, enemy_shoot_projectiles, move_enemy_projectiles,
    resolve_deflected_projectile_hits, resolve_enemy_projectiles, resolve_weapon_hits,
    start_player_attack, sync_block_weapon, sync_sheathed_weapon, tick_hit_flash,
    tick_player_attack, tick_player_hit_flash, update_player_block,
};
use crate::core::GameState;
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
                        tick_player_attack,
                        animate_weapon_swing,
                        sync_block_weapon,
                        despawn_block_weapon,
                        sync_sheathed_weapon,
                        dungeon_movement,
                        follow_camera,
                        animate_player,
                    ),
                    (
                        resolve_weapon_hits,
                        apply_enemy_contact_damage,
                        tick_hit_flash,
                        move_enemies,
                        tick_boss_attacks,
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
                    .run_if(in_state(GameState::Dungeon)),
            );
    }
}