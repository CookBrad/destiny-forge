//! Fishing VFX: held rod, line, and bobber.
//! Player body frames are driven by `animate_overworld_player` while fishing.
//!
//! Rod texture is vertical (handle at bottom, tip at top). Pivot = handle
//! ([`Anchor::BottomCenter`]). World tip = handle + dir(angle) * [`ROD_LENGTH`].
//! Line is a thin stretched sprite from tip → bobber.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::graphics::{world_transform, TILE};
use crate::overworld::layout::OverworldEntity;
use crate::overworld::movement::OverworldPlayer;
use crate::overworld::sprites::{OverworldArt, PLAYER_ANIM_FRAMES, PLAYER_SPRITE_HEIGHT};

use super::cast::ActiveCast;
use super::logic::{CastPhase, FishingAnimKind, CAST_PHASE_SECS};

/// Full pole length in world units (handle → tip). Must match sprite custom height.
pub const ROD_LENGTH: f32 = TILE * 1.9;
pub const ROD_WIDTH: f32 = TILE * 0.2;
pub const LINE_THICKNESS: f32 = 2.5;
/// Held pole angle from horizontal (degrees): tip up and out from the hand.
pub const ROD_HOLD_ANGLE_DEG: f32 = 45.0;
/// How far the bobber sits from the player into the water (world units).
pub const BOBBER_DISTANCE: f32 = TILE * 2.6;

/// Floating bobber / splash marker in the water.
#[derive(Component)]
pub struct FishingBobber;

/// Held fishing rod attached visually to the player during a cast.
#[derive(Component)]
pub struct FishingRodVisual;

/// Thin line segment from rod tip to bobber.
#[derive(Component)]
pub struct FishingLineVisual;

#[derive(Component)]
pub struct FishingAnimTimer(pub f32);

// --- pure pose helpers (unit-tested) ----------------------------------------

pub fn cast_swing_progress(remaining: f32, total: f32) -> f32 {
    let total = total.max(0.01);
    (1.0 - remaining / total).clamp(0.0, 1.0)
}

/// Body animation during cast: walk frames sell a wind-up → throw.
pub fn cast_body_frame(swing_progress: f32) -> (bool, usize) {
    let frame = ((swing_progress * PLAYER_ANIM_FRAMES as f32) as usize).min(PLAYER_ANIM_FRAMES - 1);
    (true, frame)
}

/// Held rod angle: **45° from horizontal**, tip up and out from the hand.
/// `face_right` mirrors for left-facing casters.
pub fn hold_rod_angle(face_right: bool) -> f32 {
    let a = ROD_HOLD_ANGLE_DEG.to_radians(); // π/4
    if face_right {
        a
    } else {
        std::f32::consts::PI - a
    }
}

/// Wind-up is steeper (closer to vertical) before settling to 45°.
pub fn wind_up_rod_angle(face_right: bool) -> f32 {
    let a = 75.0_f32.to_radians();
    if face_right {
        a
    } else {
        std::f32::consts::PI - a
    }
}

/// Tip angle for the current phase. Hold is always ~45° from the hand.
pub fn tip_angle_for_phase(
    kind: FishingAnimKind,
    phase: &CastPhase,
    face_right: bool,
    holding: bool,
    world_t: f32,
) -> f32 {
    let hold = hold_rod_angle(face_right);
    let wind = wind_up_rod_angle(face_right);
    match kind {
        FishingAnimKind::Cast => {
            let remaining = match phase {
                CastPhase::Casting { remaining } => *remaining,
                _ => 0.0,
            };
            let p = cast_swing_progress(remaining, CAST_PHASE_SECS);
            lerp_angle(wind, hold, p)
        }
        FishingAnimKind::Waiting => hold + (world_t * 3.0).sin() * 0.02,
        FishingAnimKind::Fighting if holding => hold + (world_t * 14.0).sin() * 0.05,
        FishingAnimKind::Fighting => hold + (world_t * 5.0).sin() * 0.02,
        FishingAnimKind::Success => {
            // Lift slightly after a catch
            let lift = if face_right { 0.35 } else { -0.35 };
            hold + lift
        }
        FishingAnimKind::Fail => hold - if face_right { 0.15 } else { -0.15 },
        FishingAnimKind::None => hold,
    }
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let mut d = b - a;
    while d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    }
    while d < -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    a + d * t
}

pub fn cast_rod_angle_right(swing_progress: f32) -> f32 {
    lerp_angle(
        wind_up_rod_angle(true),
        hold_rod_angle(true),
        swing_progress,
    )
}

pub fn hold_rod_angle_right(_kind: FishingAnimKind, _holding: bool, _t: f32) -> f32 {
    hold_rod_angle(true)
}

pub fn tip_dir(angle: f32) -> Vec2 {
    Vec2::new(angle.cos(), angle.sin())
}

pub fn rod_tip_world(handle: Vec2, angle: f32, length: f32) -> Vec2 {
    handle + tip_dir(angle) * length
}

pub fn rod_sprite_z_rotation(angle: f32) -> f32 {
    angle - std::f32::consts::FRAC_PI_2
}

/// Player feet Y from sprite-center position (center is mid-body).
pub fn player_feet_y(player_center_y: f32) -> f32 {
    player_center_y - PLAYER_SPRITE_HEIGHT * 0.5
}

/// Bobber in the water: **at feet level**, out along the cast/water direction.
pub fn bobber_at_water_level(
    player_center: Vec2,
    water_dir: Vec2,
    face_right: bool,
    distance: f32,
    wobble_x: f32,
) -> Vec2 {
    let feet_y = player_feet_y(player_center.y);
    let mut dir = water_dir;
    if dir.length_squared() < 1e-6 {
        dir = if face_right {
            Vec2::new(0.7, -0.7)
        } else {
            Vec2::new(-0.7, -0.7)
        };
    } else {
        dir = dir.normalize();
    }
    // Prefer going into the water; if aim is almost horizontal, nudge downward in XZ plane.
    if dir.y > -0.15 {
        dir.y = -0.55;
        dir = dir.normalize();
    }
    let mut pos = player_center + dir * distance;
    pos.y = feet_y;
    pos.x += wobble_x;
    pos
}

/// Midpoint, Z rotation, and length for a line sprite between two points.
/// Sprite is horizontal (length along local +X) with center pivot.
pub fn line_segment_pose(from: Vec2, to: Vec2) -> (Vec2, f32, f32) {
    let delta = to - from;
    let len = delta.length().max(1.0);
    let angle = delta.y.atan2(delta.x);
    let mid = from + delta * 0.5;
    (mid, angle, len)
}

pub fn rod_tip_local(angle: f32, length: f32) -> Vec2 {
    tip_dir(angle) * length
}

/// Apply cast/wait/fight body frames + facing onto the player sprite.
pub fn apply_player_fishing_body(
    sprite: &mut Sprite,
    art: &OverworldArt,
    kind: FishingAnimKind,
    phase: &CastPhase,
    holding: bool,
    world_t: f32,
    face_right: bool,
) {
    sprite.color = Color::WHITE;
    sprite.rect = None;
    sprite.flip_x = !face_right;

    match kind {
        FishingAnimKind::Cast => {
            let remaining = match phase {
                CastPhase::Casting { remaining } => *remaining,
                _ => 0.0,
            };
            let swing = cast_swing_progress(remaining, CAST_PHASE_SECS);
            let (walk, frame) = cast_body_frame(swing);
            sprite.image = art.player.frame_handle(walk, frame);
            let squash = 1.0 - 0.06 * (swing * std::f32::consts::PI).sin().max(0.0);
            sprite.custom_size = Some(Vec2::new(
                crate::overworld::sprites::PLAYER_SPRITE_WIDTH,
                PLAYER_SPRITE_HEIGHT * squash,
            ));
        }
        FishingAnimKind::Waiting => {
            sprite.image = art.player.frame_handle(false, 0);
            sprite.custom_size = None;
        }
        FishingAnimKind::Fighting => {
            if holding {
                let frame = ((world_t * 6.0) as usize) % 2 + 1;
                sprite.image = art.player.frame_handle(true, frame);
                sprite.custom_size = Some(Vec2::new(
                    crate::overworld::sprites::PLAYER_SPRITE_WIDTH,
                    PLAYER_SPRITE_HEIGHT * 0.96,
                ));
            } else {
                sprite.image = art.player.frame_handle(false, 0);
                sprite.custom_size = None;
            }
        }
        FishingAnimKind::Success => {
            let frame = ((world_t * 8.0) as usize) % PLAYER_ANIM_FRAMES;
            sprite.image = art.player.frame_handle(true, frame);
            sprite.color = Color::srgb(0.85, 1.0, 0.85);
            sprite.custom_size = None;
        }
        FishingAnimKind::Fail => {
            sprite.image = art.player.frame_handle(false, 0);
            sprite.color = Color::srgb(0.75, 0.75, 0.8);
            sprite.custom_size = None;
        }
        FishingAnimKind::None => {
            sprite.image = art.player.frame_handle(false, 0);
            sprite.custom_size = None;
        }
    }
}

// --- rod / line / bobber ----------------------------------------------------

pub fn sync_fishing_props(
    mut commands: Commands,
    time: Res<Time>,
    cast: Res<ActiveCast>,
    art: Option<Res<OverworldArt>>,
    player: Query<
        &Transform,
        (
            With<OverworldPlayer>,
            Without<FishingBobber>,
            Without<FishingRodVisual>,
            Without<FishingLineVisual>,
        ),
    >,
    mut rods: Query<
        (Entity, &mut Transform, &mut Sprite, &mut FishingAnimTimer),
        (
            With<FishingRodVisual>,
            Without<OverworldPlayer>,
            Without<FishingBobber>,
            Without<FishingLineVisual>,
        ),
    >,
    mut lines: Query<
        (Entity, &mut Transform, &mut Sprite),
        (
            With<FishingLineVisual>,
            Without<OverworldPlayer>,
            Without<FishingBobber>,
            Without<FishingRodVisual>,
        ),
    >,
    mut bobbers: Query<
        (Entity, &mut Transform, &mut Sprite),
        (
            With<FishingBobber>,
            Without<OverworldPlayer>,
            Without<FishingRodVisual>,
            Without<FishingLineVisual>,
        ),
    >,
) {
    let Some(art) = art else {
        return;
    };

    let kind = cast.state.anim_kind();
    let Ok(player_tf) = player.get_single() else {
        despawn_props(&mut commands, &rods, &lines, &bobbers);
        return;
    };

    if kind == FishingAnimKind::None {
        if !rods.is_empty() || !bobbers.is_empty() || !lines.is_empty() {
            despawn_props(&mut commands, &rods, &lines, &bobbers);
        }
        return;
    }

    let player_pos = player_tf.translation.truncate();
    let face_right = cast.face_right;
    let holding = cast.holding;
    let world_t = time.elapsed_secs();

    // Pole locked at ~45° from the hand (up and out), not aimed at the bobber.
    let tip_angle = tip_angle_for_phase(kind, &cast.state.phase, face_right, holding, world_t);
    let handle = player_pos + hand_offset(face_right);
    let tip = rod_tip_world(handle, tip_angle, ROD_LENGTH);

    // --- Rod ---
    if rods.is_empty() {
        spawn_rod(&mut commands, &art, handle, tip_angle);
    } else {
        for (_, mut tf, mut sprite, mut timer) in &mut rods {
            timer.0 += time.delta_secs();
            tf.translation.x = handle.x;
            tf.translation.y = handle.y;
            tf.translation.z = 5.2;
            tf.rotation = Quat::from_rotation_z(rod_sprite_z_rotation(tip_angle));
            sprite.image = art.fishing_rod.clone();
            sprite.color = rod_color(kind, holding);
            sprite.custom_size = Some(Vec2::new(ROD_WIDTH, ROD_LENGTH));
            sprite.anchor = Anchor::BottomCenter;
            sprite.flip_x = false;
        }
    }

    let show_bobber = match kind {
        FishingAnimKind::Cast => {
            let remaining = match cast.state.phase {
                CastPhase::Casting { remaining } => remaining,
                _ => 0.0,
            };
            // Show bobber/line once the cast is past the wind-up peak
            cast_swing_progress(remaining, CAST_PHASE_SECS) > 0.45
        }
        FishingAnimKind::Waiting | FishingAnimKind::Fighting | FishingAnimKind::Success => true,
        _ => false,
    };

    if show_bobber {
        let wobble = (world_t * 7.0).sin() * TILE * 0.06;
        // Bobber in the water at the player's feet height — not stuck on the tip.
        let bobber_pos = bobber_at_water_level(
            player_pos,
            cast.water_dir,
            face_right,
            BOBBER_DISTANCE,
            wobble,
        );

        // --- Bobber ---
        if bobbers.is_empty() {
            spawn_bobber(&mut commands, &art, bobber_pos, kind);
        } else {
            for (_, mut tf, mut sprite) in &mut bobbers {
                tf.translation.x = bobber_pos.x;
                tf.translation.y = bobber_pos.y;
                tf.translation.z = 5.4;
                sprite.color = bobber_color(kind, holding);
                sprite.custom_size = Some(Vec2::splat(TILE * 0.22));
            }
        }

        // --- Line tip → bobber (diagonal string, not collinear with the pole) ---
        let (mid, line_angle, line_len) = line_segment_pose(tip, bobber_pos);
        if lines.is_empty() {
            spawn_line(&mut commands, &art, mid, line_angle, line_len);
        } else {
            for (_, mut tf, mut sprite) in &mut lines {
                tf.translation.x = mid.x;
                tf.translation.y = mid.y;
                tf.translation.z = 5.3;
                tf.rotation = Quat::from_rotation_z(line_angle);
                sprite.image = art.path.clone(); // solid-ish strip
                sprite.color = Color::srgb(0.92, 0.93, 0.95);
                sprite.custom_size = Some(Vec2::new(line_len, LINE_THICKNESS));
                sprite.anchor = Anchor::Center;
            }
        }
    } else {
        for (e, _, _) in &bobbers {
            commands.entity(e).try_despawn_recursive();
        }
        for (e, _, _) in &lines {
            commands.entity(e).try_despawn_recursive();
        }
    }
}

pub fn sync_fishing_animation(
    commands: Commands,
    time: Res<Time>,
    cast: Res<ActiveCast>,
    art: Option<Res<OverworldArt>>,
    player: Query<
        &Transform,
        (
            With<OverworldPlayer>,
            Without<FishingBobber>,
            Without<FishingRodVisual>,
            Without<FishingLineVisual>,
        ),
    >,
    rods: Query<
        (Entity, &mut Transform, &mut Sprite, &mut FishingAnimTimer),
        (
            With<FishingRodVisual>,
            Without<OverworldPlayer>,
            Without<FishingBobber>,
            Without<FishingLineVisual>,
        ),
    >,
    lines: Query<
        (Entity, &mut Transform, &mut Sprite),
        (
            With<FishingLineVisual>,
            Without<OverworldPlayer>,
            Without<FishingBobber>,
            Without<FishingRodVisual>,
        ),
    >,
    bobbers: Query<
        (Entity, &mut Transform, &mut Sprite),
        (
            With<FishingBobber>,
            Without<OverworldPlayer>,
            Without<FishingRodVisual>,
            Without<FishingLineVisual>,
        ),
    >,
) {
    sync_fishing_props(commands, time, cast, art, player, rods, lines, bobbers);
}

fn despawn_props(
    commands: &mut Commands,
    rods: &Query<
        (Entity, &mut Transform, &mut Sprite, &mut FishingAnimTimer),
        (
            With<FishingRodVisual>,
            Without<OverworldPlayer>,
            Without<FishingBobber>,
            Without<FishingLineVisual>,
        ),
    >,
    lines: &Query<
        (Entity, &mut Transform, &mut Sprite),
        (
            With<FishingLineVisual>,
            Without<OverworldPlayer>,
            Without<FishingBobber>,
            Without<FishingRodVisual>,
        ),
    >,
    bobbers: &Query<
        (Entity, &mut Transform, &mut Sprite),
        (
            With<FishingBobber>,
            Without<OverworldPlayer>,
            Without<FishingRodVisual>,
            Without<FishingLineVisual>,
        ),
    >,
) {
    for (e, _, _, _) in rods.iter() {
        commands.entity(e).try_despawn_recursive();
    }
    for (e, _, _) in lines.iter() {
        commands.entity(e).try_despawn_recursive();
    }
    for (e, _, _) in bobbers.iter() {
        commands.entity(e).try_despawn_recursive();
    }
}

fn hand_offset(face_right: bool) -> Vec2 {
    let x = if face_right {
        TILE * 0.28
    } else {
        -TILE * 0.28
    };
    // Higher grip so the whole pole sits up on the character.
    Vec2::new(x, TILE * 0.28)
}

fn rod_color(kind: FishingAnimKind, holding: bool) -> Color {
    match kind {
        FishingAnimKind::Success => Color::srgb(0.7, 1.0, 0.7),
        FishingAnimKind::Fail => Color::srgb(0.55, 0.55, 0.6),
        FishingAnimKind::Fighting if holding => Color::srgb(1.0, 0.95, 0.8),
        _ => Color::WHITE,
    }
}

fn bobber_color(kind: FishingAnimKind, holding: bool) -> Color {
    match kind {
        FishingAnimKind::Fighting if holding => Color::srgb(1.0, 0.45, 0.2),
        FishingAnimKind::Success => Color::srgb(0.3, 0.95, 0.4),
        FishingAnimKind::Waiting => Color::srgb(0.95, 0.3, 0.25),
        _ => Color::srgb(0.9, 0.35, 0.25),
    }
}

fn spawn_rod(commands: &mut Commands, art: &OverworldArt, handle: Vec2, tip_angle: f32) {
    commands.spawn((
        Sprite {
            image: art.fishing_rod.clone(),
            color: Color::WHITE,
            custom_size: Some(Vec2::new(ROD_WIDTH, ROD_LENGTH)),
            anchor: Anchor::BottomCenter,
            ..default()
        },
        Transform {
            translation: handle.extend(5.2),
            rotation: Quat::from_rotation_z(rod_sprite_z_rotation(tip_angle)),
            ..default()
        },
        FishingRodVisual,
        FishingAnimTimer(0.0),
        OverworldEntity,
    ));
}

fn spawn_line(commands: &mut Commands, art: &OverworldArt, mid: Vec2, angle: f32, len: f32) {
    commands.spawn((
        Sprite {
            image: art.path.clone(),
            color: Color::srgb(0.92, 0.93, 0.95),
            custom_size: Some(Vec2::new(len, LINE_THICKNESS)),
            anchor: Anchor::Center,
            ..default()
        },
        Transform {
            translation: mid.extend(5.3),
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        FishingLineVisual,
        OverworldEntity,
    ));
}

fn spawn_bobber(commands: &mut Commands, art: &OverworldArt, pos: Vec2, kind: FishingAnimKind) {
    commands.spawn((
        Sprite {
            image: art.seed.clone(),
            color: bobber_color(kind, false),
            custom_size: Some(Vec2::splat(TILE * 0.22)),
            ..default()
        },
        world_transform(pos, 5.4),
        FishingBobber,
        OverworldEntity,
    ));
}

pub fn cleanup_fishing_animation(
    mut commands: Commands,
    bobbers: Query<Entity, With<FishingBobber>>,
    rods: Query<Entity, With<FishingRodVisual>>,
    lines: Query<Entity, With<FishingLineVisual>>,
) {
    for e in bobbers.iter().chain(rods.iter()).chain(lines.iter()) {
        commands.entity(e).try_despawn_recursive();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hold_rod_is_forty_five_degrees() {
        let right = hold_rod_angle(true);
        let left = hold_rod_angle(false);
        assert!((right.to_degrees() - 45.0).abs() < 0.1);
        assert!((left.to_degrees() - 135.0).abs() < 0.1);
        // Tip is higher than the handle
        assert!(tip_dir(right).y > 0.5);
        assert!(tip_dir(left).y > 0.5);
    }

    #[test]
    fn bobber_is_at_feet_level_not_on_tip() {
        let center = Vec2::new(100.0, 200.0);
        let feet = player_feet_y(center.y);
        let bob = bobber_at_water_level(center, Vec2::new(0.2, -1.0), true, BOBBER_DISTANCE, 0.0);
        assert!(
            (bob.y - feet).abs() < 0.01,
            "bobber y {} should match feet {}",
            bob.y,
            feet
        );
        // Out in front of the player into the water
        assert!(bob.x > center.x || bob.y < center.y);
        let tip = rod_tip_world(center + hand_offset(true), hold_rod_angle(true), ROD_LENGTH);
        assert!(
            (bob - tip).length() > TILE,
            "bobber should not sit on the pole tip"
        );
    }

    #[test]
    fn line_pose_connects_tip_to_bobber() {
        let tip = Vec2::new(10.0, 20.0);
        let bob = Vec2::new(40.0, 5.0);
        let (mid, angle, len) = line_segment_pose(tip, bob);
        assert!((mid - (tip + bob) * 0.5).length() < 0.01);
        assert!((len - tip.distance(bob)).abs() < 0.01);
        let recovered = mid + Vec2::new(angle.cos(), angle.sin()) * (len * 0.5);
        assert!((recovered - bob).length() < 0.1);
    }

    #[test]
    fn cast_swing_progress_runs_zero_to_one() {
        assert!((cast_swing_progress(CAST_PHASE_SECS, CAST_PHASE_SECS) - 0.0).abs() < 0.01);
        assert!((cast_swing_progress(0.0, CAST_PHASE_SECS) - 1.0).abs() < 0.01);
    }
}
