use bevy::prelude::*;

use crate::combat::{
    spawn_sheathed_sword, ContactDamageCooldown, EquippedWeapon, Health, PlayerAttack,
    PlayerBlock, PLAYER_MAX_HEALTH,
};
use rand::Rng;
use crate::graphics::{
    center_on_surface, scaled_transform, DungeonScrollBounds, DUNGEON_FLOOR_Y, ENEMY_DISPLAY_SIZE,
    PIXEL_SCALE, TILE,
};

use super::animation::PlayerAnimation;
use super::boss::BossAttackController;
use super::enemy::{
    DungeonProgress, EnemyContactDamage, EnemyHitbox, EnemyKind, EnemyShootCooldown,
    KingSlimeBoss, Patrol,
};
use super::generation::{generate_floor, random_seed};
use super::interaction::LadderPrompt;
use super::level::{BossSpawn, DungeonLayout, EnemySpawn, GeneratedFloor, PitfallSpec, PlatformSpec};
use super::movement::{DungeonPlayer, PlayerAirJumps, PlayerVelocity};
use super::sprites::{player_frame_rect, player_sprite_size, DungeonArt};

#[derive(Component)]
pub struct DungeonEntity;

#[derive(Component, Clone, Copy)]
pub struct PlatformCollider {
    pub min_x: f32,
    pub max_x: f32,
    pub top_y: f32,
}

#[derive(Component)]
pub struct DungeonExit;

#[derive(Component)]
pub struct Pitfall;

const BOSS_DISPLAY_SCALE: f32 = 2.0;
const BOSS_MAX_HEALTH: f32 = 120.0;

pub fn setup_dungeon(mut commands: Commands, asset_server: Res<AssetServer>) {
    let art = DungeonArt::load(&asset_server);
    let seed = random_seed();
    let floor = generate_floor(seed);

    commands.init_resource::<LadderPrompt>();
    commands.init_resource::<DungeonProgress>();
    commands.insert_resource(DungeonLayout {
        seed,
        floor: floor.clone(),
    });
    commands.insert_resource(DungeonScrollBounds {
        width: floor.width_pixels(),
    });

    spawn_backdrop(&mut commands, &art, &floor);
    for segment in &floor.ground_segments {
        spawn_ground(&mut commands, &art, *segment);
    }
    spawn_pitfalls(&mut commands, &art, &floor.pitfalls);
    for platform in &floor.platforms {
        spawn_platform(&mut commands, &art, *platform);
    }
    spawn_ladder_exit(&mut commands, &art, floor.ladder_tile);
    spawn_player(&mut commands, &art, floor.player_start_x);
    for enemy in &floor.enemies {
        spawn_enemy(&mut commands, &art, *enemy);
    }
    for bat in &floor.bats {
        spawn_enemy(
            &mut commands,
            &art,
            EnemySpawn {
                kind: EnemyKind::Bat,
                x: bat.x,
                top_y: bat.top_y,
            },
        );
    }
    spawn_king_slime(&mut commands, &art, floor.boss);

    info!("Generated dungeon floor (seed {seed}, {} tiles)", floor.width_tiles);
    commands.insert_resource(art);
}

fn spawn_backdrop(commands: &mut Commands, art: &DungeonArt, floor: &GeneratedFloor) {
    let wall = art.wall.clone();
    for row in 0..floor.backdrop_rows {
        for column in 0..floor.width_tiles {
            let position = Vec2::new(column as f32 * TILE, row as f32 * TILE);
            commands.spawn((
                Sprite {
                    image: wall.clone(),
                    ..default()
                },
                scaled_transform(position, 0.0),
                DungeonEntity,
            ));
        }
    }
}

fn spawn_ground(commands: &mut Commands, art: &DungeonArt, spec: PlatformSpec) {
    spawn_platform_tiles(commands, art, spec, true);
}

fn spawn_pitfalls(commands: &mut Commands, art: &DungeonArt, pitfalls: &[PitfallSpec]) {
    for pit in pitfalls {
        for tile in 0..pit.width_tiles {
            let x = pit.left + tile as f32 * TILE + TILE * 0.5;
            let y = DUNGEON_FLOOR_Y - TILE * 1.25;
            commands.spawn((
                Sprite {
                    image: art.floor_ground.clone(),
                    color: Color::srgba(0.12, 0.05, 0.1, 0.92),
                    ..default()
                },
                scaled_transform(Vec2::new(x, y), 0.5),
                Pitfall,
                DungeonEntity,
            ));
        }
    }
}

fn spawn_platform(commands: &mut Commands, art: &DungeonArt, spec: PlatformSpec) {
    spawn_platform_tiles(commands, art, spec, false);
}

fn spawn_platform_tiles(
    commands: &mut Commands,
    art: &DungeonArt,
    spec: PlatformSpec,
    ground: bool,
) {
    let texture = if ground {
        art.floor_ground.clone()
    } else {
        art.floor_platform.clone()
    };

    let width = spec.width_tiles as f32 * TILE;
    let collider = PlatformCollider {
        min_x: spec.left,
        max_x: spec.left + width,
        top_y: spec.top_y,
    };

    for tile in 0..spec.width_tiles {
        let x = spec.left + tile as f32 * TILE + TILE * 0.5;
        let y = spec.top_y - TILE * 0.5;
        commands.spawn((
            Sprite {
                image: texture.clone(),
                ..default()
            },
            scaled_transform(Vec2::new(x, y), 1.0),
            collider,
            DungeonEntity,
        ));
    }
}

fn spawn_ladder_exit(commands: &mut Commands, art: &DungeonArt, ladder_tile: u32) {
    let x = ladder_tile as f32 * TILE + TILE * 0.5;
    let y = DUNGEON_FLOOR_Y - TILE * 0.5;

    commands.spawn((
        Sprite {
            image: art.floor_ladder.clone(),
            ..default()
        },
        scaled_transform(Vec2::new(x, y), 1.0),
        PlatformCollider {
            min_x: x - TILE * 0.5,
            max_x: x + TILE * 0.5,
            top_y: DUNGEON_FLOOR_Y,
        },
        DungeonExit,
        DungeonEntity,
    ));
}

fn spawn_player(commands: &mut Commands, art: &DungeonArt, start_x: f32) {
    let height = player_sprite_size().y;
    let start = Vec2::new(
        start_x,
        center_on_surface(DUNGEON_FLOOR_Y, height),
    );

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
            EquippedWeapon::default(),
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

fn spawn_enemy(commands: &mut Commands, art: &DungeonArt, spec: EnemySpawn) {
    let radius = spec.kind.patrol_radius_tiles() * TILE;
    let patrol = Patrol::between(spec.x - radius, spec.x + radius, spec.kind.patrol_speed());
    let image = enemy_texture(art, spec.kind);

    let (x, y) = if spec.kind.is_airborne() {
        (spec.x, spec.top_y + 3.0 * TILE)
    } else {
        (spec.x, center_on_surface(spec.top_y, ENEMY_DISPLAY_SIZE.y))
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

fn spawn_king_slime(commands: &mut Commands, art: &DungeonArt, spec: BossSpawn) {
    let y = center_on_surface(spec.top_y, ENEMY_DISPLAY_SIZE.y);
    let boss_scale = PIXEL_SCALE * BOSS_DISPLAY_SCALE;

    commands.spawn((
        Sprite {
            image: art.slime.clone(),
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

fn despawn_dungeon(
    commands: &mut Commands,
    entities: &Query<Entity, With<DungeonEntity>>,
) {
    commands.remove_resource::<DungeonArt>();
    commands.remove_resource::<LadderPrompt>();
    commands.remove_resource::<DungeonProgress>();
    commands.remove_resource::<DungeonScrollBounds>();
    commands.remove_resource::<DungeonLayout>();
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn cleanup_dungeon(
    mut commands: Commands,
    entities: Query<Entity, With<DungeonEntity>>,
) {
    despawn_dungeon(&mut commands, &entities);
}

pub fn retry_dungeon(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    entities: Query<Entity, With<DungeonEntity>>,
    mut next_play: ResMut<NextState<crate::core::DungeonPlayState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        despawn_dungeon(&mut commands, &entities);
        setup_dungeon(commands, asset_server);
        next_play.set(crate::core::DungeonPlayState::Running);
    }
}