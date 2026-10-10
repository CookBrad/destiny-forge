use bevy::prelude::*;

use crate::overworld::movement::OverworldPlayer;
use crate::overworld::sprites::PLAYER_NON_COMBAT_ROOT;

use super::layout::MineEntity;

pub const PICKAXE_SWING_SECONDS: f32 = 0.3;
pub const DWARF_PICKAXE_SWING_FRAME_COUNT: usize = 4;

#[derive(Resource, Clone)]
pub struct DwarfPickaxeSwingFrames(pub [Handle<Image>; DWARF_PICKAXE_SWING_FRAME_COUNT]);

impl DwarfPickaxeSwingFrames {
    pub fn load(asset_server: &AssetServer) -> Self {
        Self(std::array::from_fn(|frame| {
            asset_server.load(format!(
                "{PLAYER_NON_COMBAT_ROOT}/dwarf_m_pickaxe_swing_f{frame}.png"
            ))
        }))
    }
}

#[derive(Component)]
pub struct PickaxeSwing {
    pub timer: Timer,
    pub target_node: Entity,
}

#[derive(Event)]
pub struct PickaxeSwingLanded {
    pub target_node: Entity,
}

pub fn swing_progress(swing: &PickaxeSwing) -> f32 {
    swing.timer.fraction().clamp(0.0, 1.0)
}

pub fn dwarf_pickaxe_swing_frame_index(progress: f32) -> usize {
    let frame = (progress.clamp(0.0, 1.0) * DWARF_PICKAXE_SWING_FRAME_COUNT as f32) as usize;
    frame.min(DWARF_PICKAXE_SWING_FRAME_COUNT - 1)
}

pub fn spawn_pickaxe_swing(commands: &mut Commands, target_node: Entity) {
    commands.spawn((
        PickaxeSwing {
            timer: Timer::from_seconds(PICKAXE_SWING_SECONDS, TimerMode::Once),
            target_node,
        },
        MineEntity,
    ));
}

pub fn animate_pickaxe_swing(
    time: Res<Time>,
    swing_frames: Res<DwarfPickaxeSwingFrames>,
    mut commands: Commands,
    mut landed_swings: EventWriter<PickaxeSwingLanded>,
    mut player_sprites: Query<&mut Sprite, With<OverworldPlayer>>,
    mut swings: Query<(Entity, &mut PickaxeSwing)>,
) {
    for (swing_entity, mut swing) in &mut swings {
        swing.timer.tick(time.delta());
        let frame = dwarf_pickaxe_swing_frame_index(swing_progress(&swing));
        if let Ok(mut player_sprite) = player_sprites.get_single_mut() {
            player_sprite.image = swing_frames.0[frame].clone();
            player_sprite.rect = None;
        }
        if swing.timer.just_finished() {
            landed_swings.send(PickaxeSwingLanded {
                target_node: swing.target_node,
            });
            commands.entity(swing_entity).try_despawn_recursive();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swing_frames_run_raised_to_strike_in_order_and_hold_the_strike_at_the_end() {
        let frames_at_each_step: Vec<usize> = (0..=20)
            .map(|step| dwarf_pickaxe_swing_frame_index(step as f32 / 20.0))
            .collect();
        assert_eq!(frames_at_each_step.first(), Some(&0));
        assert_eq!(
            frames_at_each_step.last(),
            Some(&(DWARF_PICKAXE_SWING_FRAME_COUNT - 1))
        );
        assert!(frames_at_each_step
            .windows(2)
            .all(|pair| pair[1] >= pair[0]));
        for frame in 0..DWARF_PICKAXE_SWING_FRAME_COUNT {
            assert!(frames_at_each_step.contains(&frame));
        }
    }

    #[test]
    fn every_swing_frame_png_is_a_distinct_in_house_sprite_centred_on_the_dwarf() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join(PLAYER_NON_COMBAT_ROOT);
        let frame_bytes: Vec<Vec<u8>> = (0..DWARF_PICKAXE_SWING_FRAME_COUNT)
            .map(|frame| {
                std::fs::read(folder.join(format!("dwarf_m_pickaxe_swing_f{frame}.png")))
                    .expect("swing frame png is committed")
            })
            .collect();
        for (index, bytes) in frame_bytes.iter().enumerate() {
            let png_width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
            let png_height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
            assert_eq!((png_width, png_height), (48, 60), "frame {index}");
            for later in &frame_bytes[index + 1..] {
                assert_ne!(bytes, later);
            }
        }
    }
}
