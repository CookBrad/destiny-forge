use bevy::prelude::*;

use rand::Rng;

use crate::combat::{ContactDamageCooldown, Health, PlayerAttack, PlayerBlock, PLAYER_MAX_HEALTH};
use crate::graphics::{
    center_on_surface, sole_anchor, world_transform, DUNGEON_FLOOR_Y, ENEMY_DISPLAY_SIZE,
    HUNTER_BODY_PX, KING_SLIME_CANVAS_PX, KING_SLIME_GAMEPLAY_SCALE, TILE,
};
use crate::player::Loadout;

use super::super::animation::PlayerAnimation;
use super::super::boss::BossAttackController;
use super::super::enemy::{
    EnemyContactDamage, EnemyHitbox, EnemyKind, EnemyShootCooldown, GoblinJump, KingSlimeBoss,
    Patrol, SpriteCanvas,
};
use super::super::enemy_anim::{enemy_cell_rect, EnemyAnimation};
use super::super::hunter_pose::{playback_for, pose_anchor, sheet_for, HunterPose};
use super::super::level::{
    ground_patrol_range, BossSpawn, EnemySpawn, GeneratedFloor, PlatformSpec,
};
use super::super::movement::{DungeonPlayer, PlayerAirJumps, PlayerVelocity};
use super::super::sprites::{enemy_sheet_px, player_frame_rect, DungeonArt};
use super::DungeonEntity;

const BOSS_MAX_HEALTH: f32 = 120.0;

pub fn spawn_player(commands: &mut Commands, art: &DungeonArt, start_x: f32, loadout: &Loadout) {
    let start = Vec2::new(
        start_x,
        center_on_surface(DUNGEON_FLOOR_Y, HUNTER_BODY_PX.y),
    );

    let idle = HunterPose::Idle;
    commands.spawn((
        Sprite {
            image: art.hunter_image(sheet_for(idle)),
            rect: Some(player_frame_rect(playback_for(idle).origin_cell)),
            anchor: pose_anchor(idle),
            ..default()
        },
        world_transform(start, 10.0),
        DungeonPlayer,
        PlayerVelocity::default(),
        PlayerAirJumps::default(),
        PlayerAnimation::default(),
        loadout.equipped_weapon(),
        PlayerAttack::inactive(),
        PlayerBlock::default(),
        Health::new(PLAYER_MAX_HEALTH),
        ContactDamageCooldown::default(),
        DungeonEntity,
    ));
}

pub fn spawn_enemies(commands: &mut Commands, art: &DungeonArt, floor: &GeneratedFloor) {
    for enemy in &floor.enemies {
        spawn_enemy(commands, art, *enemy, &floor.ground_segments);
    }
    for bat in &floor.bats {
        spawn_enemy(
            commands,
            art,
            EnemySpawn {
                kind: EnemyKind::Bat,
                x: bat.x,
                top_y: bat.top_y,
            },
            &floor.ground_segments,
        );
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    art: &DungeonArt,
    spec: EnemySpawn,
    ground_segments: &[PlatformSpec],
) {
    let radius = spec.kind.patrol_radius_tiles() * TILE;
    let (patrol_min, patrol_max) = if spec.kind.is_airborne() {
        (spec.x - radius, spec.x + radius)
    } else if let Some((min_x, max_x)) = ground_patrol_range(spec.x, ground_segments) {
        (min_x, max_x)
    } else {
        (spec.x - radius, spec.x + radius)
    };
    let patrol = Patrol::between(patrol_min, patrol_max, spec.kind.patrol_speed());
    let image = enemy_texture(art, spec.kind);
    let canvas = canvas_for(spec.kind);

    let (x, y) = if spec.kind.is_airborne() {
        (spec.x, spec.top_y + 3.0 * TILE)
    } else {
        (spec.x, center_on_surface(spec.top_y, ENEMY_DISPLAY_SIZE.y))
    };

    let mut entity = commands.spawn((
        authored_sprite(image, canvas),
        world_transform(Vec2::new(x, y), 5.0),
        spec.kind,
        EnemyHitbox::standard(),
        canvas,
        Health::new(spec.kind.max_health()),
        EnemyContactDamage(spec.kind.contact_damage()),
        patrol,
        EnemyAnimation::standing_at(x),
        DungeonEntity,
    ));

    if spec.kind.shoots_projectiles() {
        let delay = rand::thread_rng().gen_range(0.5..spec.kind.shoot_cooldown());
        entity.insert(EnemyShootCooldown(Timer::from_seconds(
            delay,
            TimerMode::Once,
        )));
    }

    if spec.kind == EnemyKind::Goblin {
        entity.insert(GoblinJump::default());
    }
}

fn enemy_texture(art: &DungeonArt, kind: EnemyKind) -> Handle<Image> {
    match kind {
        EnemyKind::Slime => art.slime.clone(),
        EnemyKind::Bat => art.bat.clone(),
        EnemyKind::Goblin => art.goblin.clone(),
        EnemyKind::Skeleton => art.skeleton.clone(),
        EnemyKind::Zombie => art.zombie.clone(),
    }
}

fn canvas_for(kind: EnemyKind) -> SpriteCanvas {
    let size = enemy_sheet_px(kind);
    if kind.is_airborne() {
        SpriteCanvas::centered(size)
    } else {
        SpriteCanvas::grounded(size, ENEMY_DISPLAY_SIZE.y)
    }
}

/// One cell of the strip. The anchor pins the sole; scale stays 1.
fn authored_sprite(image: Handle<Image>, canvas: SpriteCanvas) -> Sprite {
    Sprite {
        image,
        rect: Some(enemy_cell_rect(0, canvas.size)),
        anchor: sole_anchor(canvas.size.y, canvas.sole_below_origin),
        ..default()
    }
}

pub fn spawn_king_slime(commands: &mut Commands, art: &DungeonArt, spec: BossSpawn) {
    let gameplay_height = ENEMY_DISPLAY_SIZE.y * KING_SLIME_GAMEPLAY_SCALE;
    let y = center_on_surface(spec.top_y, gameplay_height);
    let canvas = SpriteCanvas::grounded(KING_SLIME_CANVAS_PX, gameplay_height);

    commands.spawn((
        authored_sprite(art.king_slime.clone(), canvas),
        world_transform(Vec2::new(spec.x, y), 6.0),
        KingSlimeBoss,
        BossAttackController::new(),
        EnemyHitbox::scaled(KING_SLIME_GAMEPLAY_SCALE),
        canvas,
        Health::new(BOSS_MAX_HEALTH),
        EnemyContactDamage(12.0),
        Patrol::between(spec.patrol_min_x, spec.patrol_max_x, 22.0),
        EnemyAnimation::standing_at(spec.x),
        DungeonEntity,
    ));
}
