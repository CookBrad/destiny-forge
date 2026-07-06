use bevy::prelude::*;

use crate::audio::CombatSfx;
use crate::dungeon::{DungeonArt, DungeonPlayer, PlayerAnimation, PlayerVelocity, PLAYER_IDLE_FRAMES, PLAYER_RUN_FRAMES};

use super::attack::PlayerAttack;
use super::player_block::PlayerBlock;
use super::skills::{SkillBindings, SkillKind};
use super::special_moves::PlayerSpecialMove;
use super::weapon::{EquippedWeapon, WeaponKind};

#[derive(Component)]
pub struct WeaponBlockFx;

const BLOCK_SWORD_X: f32 = 5.0;
const BLOCK_SWORD_Y: f32 = -4.0;
const BLOCK_SWORD_Z: f32 = 0.5;
const BLOCK_SWORD_ANGLE: f32 = -0.55;

const IDLE_BLOCK_BOB: [f32; 4] = [0.0, -0.5, -1.0, -0.5];
const RUN_BLOCK_BOB: [f32; 4] = [-1.5, 0.5, 1.5, -1.0];

pub fn update_player_block(
    bindings: Res<SkillBindings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sfx: EventWriter<CombatSfx>,
    mut player: Query<
        (&PlayerAttack, &mut PlayerBlock, Option<&PlayerSpecialMove>),
        With<DungeonPlayer>,
    >,
) {
    let Ok((attack, mut block, special)) = player.get_single_mut() else {
        return;
    };

    if attack.is_active() || special.is_some_and(|m| m.is_active()) {
        block.active = false;
        return;
    }

    if SkillBindings::skill_just_pressed(&keyboard, &bindings, SkillKind::Block) {
        sfx.send(CombatSfx::Block);
    }

    block.active = SkillBindings::skill_pressed(&keyboard, &bindings, SkillKind::Block);
}

pub fn sync_block_weapon(
    mut commands: Commands,
    art: Res<DungeonArt>,
    player: Query<
        (
            Entity,
            &EquippedWeapon,
            &PlayerBlock,
            &PlayerAttack,
            Option<&PlayerSpecialMove>,
            &PlayerAnimation,
            &PlayerVelocity,
        ),
        With<DungeonPlayer>,
    >,
    mut blocks: Query<&mut Transform, With<WeaponBlockFx>>,
) {
    let Ok((entity, weapon, block, attack, special, animation, velocity)) = player.get_single() else {
        return;
    };

    let show = block.is_active()
        && !attack.is_active()
        && !special.is_some_and(|m| m.is_active())
        && matches!(weapon.0, WeaponKind::RustySword | WeaponKind::IronSword);
    let bob = block_bob_offset(animation, velocity);
    let pose = block_pose(bob);

    if !show {
        return;
    }

    if blocks.is_empty() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Sprite {
                    image: art.weapon_anime_sword.clone(),
                    ..default()
                },
                Transform {
                    translation: pose.translation,
                    rotation: pose.rotation,
                    ..default()
                },
                WeaponBlockFx,
            ));
        });
    } else {
        for mut transform in &mut blocks {
            transform.translation = pose.translation;
            transform.rotation = pose.rotation;
        }
    }
}

pub fn despawn_block_weapon(
    mut commands: Commands,
    player: Query<&PlayerBlock, With<DungeonPlayer>>,
    blocks: Query<Entity, With<WeaponBlockFx>>,
) {
    let Ok(block) = player.get_single() else {
        return;
    };

    if !block.is_active() {
        for entity in &blocks {
            commands.entity(entity).try_despawn();
        }
    }
}

struct BlockPose {
    translation: Vec3,
    rotation: Quat,
}

fn block_pose(bob: f32) -> BlockPose {
    BlockPose {
        translation: Vec3::new(BLOCK_SWORD_X, BLOCK_SWORD_Y + bob, BLOCK_SWORD_Z),
        rotation: Quat::from_rotation_z(BLOCK_SWORD_ANGLE),
    }
}

fn block_bob_offset(animation: &PlayerAnimation, velocity: &PlayerVelocity) -> f32 {
    if !velocity.grounded {
        return 0.0;
    }

    if velocity.x.abs() > 1.0 {
        let frame = animation.frame % PLAYER_RUN_FRAMES;
        return RUN_BLOCK_BOB[frame];
    }

    let frame = animation.frame % PLAYER_IDLE_FRAMES;
    IDLE_BLOCK_BOB[frame]
}