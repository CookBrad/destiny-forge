use bevy::prelude::*;

use crate::core::GameState;

use super::carve::carve_nearby_corpses;
use super::enemy::{
    enemy_chase_player, enemy_contact_damage, enemy_patrol, spawn_corpse_on_death,
};
use crate::graphics::{animate_sprites, update_dungeon_player_animation};

use super::movement::{apply_platform_collisions, dungeon_movement};
use super::setup::{cleanup_dungeon, setup_dungeon};
use super::transition::leave_dungeon_via_exit;

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Dungeon), setup_dungeon)
            .add_systems(OnExit(GameState::Dungeon), cleanup_dungeon)
            .add_systems(
                Update,
                (
                    dungeon_movement,
                    apply_platform_collisions,
                    enemy_patrol,
                    enemy_chase_player,
                    enemy_contact_damage,
                    spawn_corpse_on_death,
                    carve_nearby_corpses,
                    leave_dungeon_via_exit,
                    update_dungeon_player_animation,
                    animate_sprites,
                )
                    .chain()
                    .run_if(in_state(GameState::Dungeon)),
            );
    }
}