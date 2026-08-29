use bevy::prelude::*;

use rand::Rng;

use crate::combat::{
    spawn_sheathed_sword, ContactDamageCooldown, Health, PlayerAttack, PlayerBlock, PLAYER_MAX_HEALTH,
};
use crate::graphics::{
    center_on_surface, scaled_transform, DUNGEON_FLOOR_Y, ENEMY_DISPLAY_SIZE, PIXEL_SCALE, TILE,
};
use crate::player::Loadout;

use super::DungeonEntity;
use super::super::animation::PlayerAnimation;
use super::super::boss::BossAttackController;
use super::super::enemy::{
    EnemyContactDamage, EnemyHitbox, EnemyKind, EnemyShootCooldown, GoblinJump, KingSlimeBoss, Patrol,
};
use super::super::level::{ground_patrol_range, BossSpawn, EnemySpawn, GeneratedFloor, PlatformSpec};
use super::super::movement::{DungeonPlayer, PlayerAirJumps, PlayerVelocity};
use super::super::sprites::{player_frame_rect, player_sprite_size, slime_sprite_size, DungeonArt};

const BOSS_DISPLAY_SCALE: f32 = 2.0;
const BOSS_MAX_HEALTH: f32 = 120.0;

pub fn spawn_player(commands: &mut Commands, art: &DungeonArt, start_x: f32, loadout: &Loadout) {
    let height = player_sprite_size().y;
    let start = Vec2::new(start_x, center_on_surface(DUNGEON_FLOOR_Y, height));

    commands
        .spawn((
            Sprite {
                image: art.player_idle.clone(),
                rect: Some(player_frame_rect(0)),
                ..default()
            },
            scaled_transform(start, 10.0),
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
        ))
        .with_children(|parent| {
            parent.spawn(spawn_sheathed_sword(art.weapon_anime_sword.clone()));
        });
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

    let sprite_h = if spec.kind == EnemyKind::Slime {
        slime_sprite_size().y
    } else {
        ENEMY_DISPLAY_SIZE.y
    };
    let (x, y) = if spec.kind.is_airborne() {
        (spec.x, spec.top_y + 3.0 * TILE)
    } else {
        (spec.x, center_on_surface(spec.top_y, sprite_h))
    };

    let mut entity = commands.spawn((
        Sprite {
            image,
            ..default()
        },
        scaled_transform(Vec2::new(x, y), 5.0),
        spec.kind,
        EnemyHitbox::standard(),
        Health::new(spec.kind.max_health()),
        EnemyContactDamage(spec.kind.contact_damage()),
        patrol,
        DungeonEntity,
    ));

    if spec.kind.shoots_projectiles() {
        let delay = rand::thread_rng().gen_range(0.5..spec.kind.shoot_cooldown());
        entity.insert(EnemyShootCooldown(Timer::from_seconds(delay, TimerMode::Once)));
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

pub fn spawn_king_slime(commands: &mut Commands, art: &DungeonArt, spec: BossSpawn) {
    let y = center_on_surface(spec.top_y, ENEMY_DISPLAY_SIZE.y);
    let boss_scale = PIXEL_SCALE * BOSS_DISPLAY_SCALE;

    commands.spawn((
        Sprite {
            image: art.slime_king.clone(),
            color: Color::srgb(0.55, 0.95, 0.45),
            ..default()
        },
        Transform {
            translation: Vec3::new(spec.x, y, 6.0),
            scale: Vec3::splat(boss_scale),
            ..default()
        },
        KingSlimeBoss,
        BossAttackController::new(),
        EnemyHitbox::scaled(BOSS_DISPLAY_SCALE),
        Health::new(BOSS_MAX_HEALTH),
        EnemyContactDamage(12.0),
        Patrol::between(spec.patrol_min_x, spec.patrol_max_x, 22.0),
        DungeonEntity,
    ));
}
