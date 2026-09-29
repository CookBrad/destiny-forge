use bevy::prelude::*;

use crate::audio::CombatSfx;
use crate::dungeon::DungeonPlayer;

use super::attack::PlayerAttack;
use super::hit_stop::HitStop;
use super::player_block::PlayerBlock;
use super::skills::{SkillBindings, SkillKind};
use super::special_moves::PlayerSpecialMove;

pub fn update_player_block(
    time: Res<Time>,
    bindings: Res<SkillBindings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    hit_stop: Res<HitStop>,
    mut sfx: EventWriter<CombatSfx>,
    mut player: Query<
        (&PlayerAttack, &mut PlayerBlock, Option<&PlayerSpecialMove>),
        With<DungeonPlayer>,
    >,
) {
    if hit_stop.is_active() {
        return;
    }

    let Ok((attack, mut block, special)) = player.get_single_mut() else {
        return;
    };

    if attack.is_active() || special.is_some_and(|m| m.is_active()) {
        block.active = false;
        return;
    }

    if SkillBindings::skill_just_pressed(&keyboard, &bindings, SkillKind::Block) {
        sfx.send(CombatSfx::Block);
        block.begin_parry_window();
    }

    block.active = SkillBindings::skill_pressed(&keyboard, &bindings, SkillKind::Block);
    if block.active {
        block.tick_parry(time.delta());
    }
}
