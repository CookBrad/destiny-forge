use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use crate::core::{PlayerProfile, ProfileDirty, ToolEnergy};
use crate::graphics::world_transform;
use crate::items::{Inventory, MaterialId};
use crate::overworld::layout::tile_center;
use crate::overworld::movement::OverworldPlayer;

use super::interaction::{land_pickaxe_swing_on_ore_node, start_pickaxe_swing_at_nearest_ore};
use super::layout::OreNode;
use super::ore::OreKind;
use super::swing::{
    animate_pickaxe_swing, DwarfPickaxeSwingFrames, PickaxeSwing, PickaxeSwingLanded,
    DWARF_PICKAXE_SWING_FRAME_COUNT, PICKAXE_SWING_SECONDS,
};

const FRAME: Duration = Duration::from_millis(50);

fn distinct_swing_frame_handles() -> DwarfPickaxeSwingFrames {
    DwarfPickaxeSwingFrames(std::array::from_fn(|frame| {
        Handle::weak_from_u128(0x5_17A9_0000 + frame as u128)
    }))
}

fn headless_mine_with_copper_node_beside_player() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(FRAME))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<ToolEnergy>()
        .init_resource::<PlayerProfile>()
        .init_resource::<ProfileDirty>()
        .insert_resource(Inventory::with_starter_kit())
        .insert_resource(distinct_swing_frame_handles())
        .add_event::<PickaxeSwingLanded>()
        .add_systems(
            Update,
            (
                start_pickaxe_swing_at_nearest_ore,
                animate_pickaxe_swing,
                land_pickaxe_swing_on_ore_node,
            )
                .chain(),
        );
    app.world_mut().spawn((
        world_transform(tile_center(5, 5), 5.0),
        Sprite::default(),
        OverworldPlayer,
    ));
    let copper_node = app
        .world_mut()
        .spawn((
            world_transform(tile_center(6, 5), 2.0),
            Sprite::default(),
            OreNode {
                id: 0,
                kind: OreKind::Copper,
                hits_taken: 0,
            },
        ))
        .id();
    app.update();
    (app, copper_node)
}

fn press_space_for_one_frame(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    app.update();
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(KeyCode::Space);
    keys.clear();
}

fn swings_in_progress(app: &mut App) -> usize {
    app.world_mut()
        .query::<&PickaxeSwing>()
        .iter(app.world())
        .count()
}

fn player_swing_frame_shown(app: &mut App) -> Option<usize> {
    let shown = app
        .world_mut()
        .query_filtered::<&Sprite, With<OverworldPlayer>>()
        .single(app.world())
        .image
        .clone();
    distinct_swing_frame_handles()
        .0
        .iter()
        .position(|frame| *frame == shown)
}

fn hits_taken(app: &App, node: Entity) -> Option<u32> {
    app.world().get::<OreNode>(node).map(|node| node.hits_taken)
}

fn frames_for_one_swing() -> u32 {
    (PICKAXE_SWING_SECONDS / FRAME.as_secs_f32()).ceil() as u32 + 1
}

fn finish_swing(app: &mut App) {
    for _ in 0..frames_for_one_swing() {
        app.update();
    }
}

#[test]
fn space_starts_a_visible_swing_and_the_hit_lands_only_when_the_swing_finishes() {
    let (mut app, copper_node) = headless_mine_with_copper_node_beside_player();

    press_space_for_one_frame(&mut app);
    assert_eq!(swings_in_progress(&mut app), 1);
    assert_eq!(hits_taken(&app, copper_node), Some(0));

    finish_swing(&mut app);
    assert_eq!(swings_in_progress(&mut app), 0);
    assert_eq!(hits_taken(&app, copper_node), Some(1));
}

#[test]
fn second_swing_breaks_copper_after_its_animation_and_drops_ore() {
    let (mut app, copper_node) = headless_mine_with_copper_node_beside_player();

    press_space_for_one_frame(&mut app);
    finish_swing(&mut app);
    press_space_for_one_frame(&mut app);
    assert!(app.world().get_entity(copper_node).is_ok());

    finish_swing(&mut app);
    assert!(app.world().get_entity(copper_node).is_err());
    assert_eq!(
        app.world()
            .resource::<Inventory>()
            .count(MaterialId::CopperOre),
        1
    );
    assert_eq!(
        app.world()
            .resource::<PlayerProfile>()
            .depleted_ore_node_ids,
        vec![0]
    );
}

#[test]
fn pressing_space_mid_swing_does_not_stack_a_second_swing() {
    let (mut app, copper_node) = headless_mine_with_copper_node_beside_player();

    press_space_for_one_frame(&mut app);
    press_space_for_one_frame(&mut app);
    assert_eq!(swings_in_progress(&mut app), 1);

    finish_swing(&mut app);
    assert_eq!(hits_taken(&app, copper_node), Some(1));
}

#[test]
fn player_sprite_steps_through_every_pickaxe_swing_frame_before_the_hit_lands() {
    let (mut app, copper_node) = headless_mine_with_copper_node_beside_player();
    assert_eq!(player_swing_frame_shown(&mut app), None);

    press_space_for_one_frame(&mut app);
    let mut frames_shown = Vec::new();
    while swings_in_progress(&mut app) > 0 {
        if let Some(frame) = player_swing_frame_shown(&mut app) {
            if frames_shown.last() != Some(&frame) {
                frames_shown.push(frame);
            }
        }
        if frames_shown.last() != Some(&(DWARF_PICKAXE_SWING_FRAME_COUNT - 1)) {
            assert_eq!(hits_taken(&app, copper_node), Some(0));
        }
        app.update();
    }

    assert_eq!(
        frames_shown,
        (0..DWARF_PICKAXE_SWING_FRAME_COUNT).collect::<Vec<_>>()
    );
    assert_eq!(hits_taken(&app, copper_node), Some(1));
}

#[test]
fn swing_faces_the_dwarf_toward_an_ore_node_on_the_left() {
    let (mut app, copper_node) = headless_mine_with_copper_node_beside_player();
    app.world_mut()
        .get_mut::<Transform>(copper_node)
        .unwrap()
        .translation = world_transform(tile_center(4, 5), 2.0).translation;

    press_space_for_one_frame(&mut app);
    let player_flipped = app
        .world_mut()
        .query_filtered::<&Sprite, With<OverworldPlayer>>()
        .single(app.world())
        .flip_x;
    assert!(player_flipped);
}
