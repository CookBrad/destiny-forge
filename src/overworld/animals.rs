use bevy::prelude::*;
use rand::Rng;

use crate::graphics::TILE;

use super::layout::OverworldEntity;
use super::sprites::animal_frame_rect;

pub const PEN_MIN: Vec2 = Vec2::new(33.0 * TILE, 7.0 * TILE);
pub const PEN_MAX: Vec2 = Vec2::new(48.0 * TILE, 17.0 * TILE);
const ANIMAL_MARGIN: f32 = TILE * 0.6;
pub const WANDER_SPEED: f32 = 22.0;

#[derive(Component)]
pub struct FarmAnimal;

#[derive(Component)]
pub struct AnimalWander {
    pub direction: Vec2,
    pub speed: f32,
    pub graze_timer: Timer,
    pub grazing: bool,
}

impl AnimalWander {
    pub fn new(speed: f32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            direction: random_direction(&mut rng),
            speed,
            graze_timer: Timer::from_seconds(rng.gen_range(2.0..5.0), TimerMode::Once),
            grazing: false,
        }
    }
}

pub fn spawn_farm_animal(
    commands: &mut Commands,
    image: Handle<Image>,
    position: Vec2,
    z: f32,
    sprite_index: usize,
    wander: AnimalWander,
) -> Entity {
    let sprite_rect = animal_frame_rect(sprite_index);
    commands
        .spawn((
            Sprite {
                image,
                rect: Some(sprite_rect),
                ..default()
            },
            Transform {
                translation: position.extend(z),
                scale: Vec3::splat(crate::graphics::PIXEL_SCALE),
                ..default()
            },
            FarmAnimal,
            wander,
            OverworldEntity,
        ))
        .id()
}

pub fn move_farm_animals(
    time: Res<Time>,
    mut animals: Query<(&mut Transform, &mut AnimalWander), With<FarmAnimal>>,
) {
    let mut rng = rand::thread_rng();
    let dt = time.delta_secs();
    let bounds = pen_bounds();

    for (mut transform, mut wander) in &mut animals {
        wander.graze_timer.tick(time.delta());

        if wander.grazing {
            if wander.graze_timer.finished() {
                wander.grazing = false;
                wander.direction = random_direction(&mut rng);
                wander.graze_timer =
                    Timer::from_seconds(rng.gen_range(2.0..5.0), TimerMode::Once);
            }
            continue;
        }

        let mut pos = transform.translation.truncate();
        pos += wander.direction * wander.speed * dt;

        let mut bounced = false;
        if pos.x < bounds.min.x {
            pos.x = bounds.min.x;
            wander.direction.x = wander.direction.x.abs();
            bounced = true;
        } else if pos.x > bounds.max.x {
            pos.x = bounds.max.x;
            wander.direction.x = -wander.direction.x.abs();
            bounced = true;
        }
        if pos.y < bounds.min.y {
            pos.y = bounds.min.y;
            wander.direction.y = wander.direction.y.abs();
            bounced = true;
        } else if pos.y > bounds.max.y {
            pos.y = bounds.max.y;
            wander.direction.y = -wander.direction.y.abs();
            bounced = true;
        }

        if bounced {
            wander.direction = wander.direction.normalize_or_zero();
        }

        if wander.graze_timer.finished() {
            wander.grazing = true;
            wander.graze_timer = Timer::from_seconds(rng.gen_range(0.8..2.2), TimerMode::Once);
            wander.direction = Vec2::ZERO;
            continue;
        }

        transform.translation.x = pos.x;
        transform.translation.y = pos.y;

        if wander.direction.x.abs() > 0.01 {
            let scale = transform.scale.x.abs();
            transform.scale.x = if wander.direction.x < 0.0 { -scale } else { scale };
        }
    }
}

fn pen_bounds() -> Rect {
    Rect {
        min: PEN_MIN + Vec2::splat(ANIMAL_MARGIN),
        max: PEN_MAX - Vec2::splat(ANIMAL_MARGIN),
    }
}

fn random_direction(rng: &mut impl Rng) -> Vec2 {
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    Vec2::new(angle.cos(), angle.sin()).normalize_or_zero()
}