use bevy::prelude::*;

use crate::combat::{
    animate_weapon_swing, apply_enemy_contact_damage, resolve_weapon_hits, start_player_attack,
    tick_hit_flash, tick_player_attack,
};
use crate::core::GameState;

use super::animation::animate_player;
use super::enemy::{patrol_enemies, track_boss_defeat};
use super::interaction::{ladder_interaction, update_ladder_prompt};
use super::movement::dungeon_movement;
use super::setup::{cleanup_dungeon, setup_dungeon};

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Dungeon), setup_dungeon)
            .add_systems(OnExit(GameState::Dungeon), cleanup_dungeon)
            .add_systems(
                Update,
                (
                    start_player_attack,
                    tick_player_attack,
                    animate_weapon_swing,
                    dungeon_movement,
                    animate_player,
                    resolve_weapon_hits,
                    apply_enemy_contact_damage,
                    tick_hit_flash,
                    patrol_enemies,
                    track_boss_defeat,
                    update_ladder_prompt,
                    ladder_interaction,
                )
                    .chain()
                    .run_if(in_state(GameState::Dungeon)),
            );
    }
}