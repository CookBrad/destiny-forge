use bevy::prelude::*;
use rand::Rng;

use crate::graphics::{center_on_surface, world_transform, TILE};

use super::layout::OverworldEntity;
use super::sprites::{
    animal_frame_index, AnimalKind, ANIMAL_DISPLAY_SIZE, PLAYER_SPRITE_HEIGHT,
};

pub const PEN_MIN: Vec2 = Vec2::new(33.0 * TILE, 7.0 * TILE);
pub const PEN_MAX: Vec2 = Vec2::new(48.0 * TILE, 17.0 * TILE);
const ANIMAL_MARGIN: f32 = TILE * 0.6;
pub const WANDER_SPEED: f32 = 88.0;

#[derive(Component)]
pub struct FarmAnimal;

#[derive(Component, Clone, Copy, Debug)]
pub struct FarmAnimalSpecies(pub AnimalKind);

#[derive(Component)]
pub struct AnimalWander {
    pub direction: Vec2,
    pub speed: f32,
    pub graze_timer: Timer,
    pub grazing: bool,
    pub anim_frame: usize,
    pub anim_timer: Timer,
}

impl AnimalWander {
    pub fn new(speed: f32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            direction: random_direction(&mut rng),
            speed,
            graze_timer: Timer::from_seconds(rng.gen_range(2.0..5.0), TimerMode::Once),
            grazing: false,
            anim_frame: 0,
            anim_timer: Timer::from_seconds(0.18, TimerMode::Repeating),
        }
    }
}

pub fn spawn_farm_animal(
    commands: &mut Commands,
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    position: Vec2,
    z: f32,
    kind: AnimalKind,
    wander: AnimalWander,
) -> Entity {
    let ground_y = position.y - TILE * 0.5;
    let center = Vec2::new(
        position.x,
        center_on_surface(ground_y, PLAYER_SPRITE_HEIGHT),
    );

    commands
        .spawn((
            Sprite {
                image,
                texture_atlas: Some(TextureAtlas {
                    layout,
                    index: animal_frame_index(0),
                }),
                custom_size: Some(ANIMAL_DISPLAY_SIZE),
                ..default()
            },
            world_transform(center, z),
            FarmAnimal,
            FarmAnimalSpecies(kind),
            wander,
            OverworldEntity,
        ))
        .id()
}

pub fn move_farm_animals(
    time: Res<Time>,
    mut animals: Query<(&mut Transform, &mut Sprite, &mut AnimalWander), With<FarmAnimal>>,
) {
    let mut rng = rand::thread_rng();
    let dt = time.delta_secs();
    let bounds = pen_bounds();

    for (mut transform, mut sprite, mut wander) in &mut animals {
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
            sprite.flip_x = wander.direction.x < 0.0;
        }

        // Advance walk-cycle frame while moving.
        wander.anim_timer.tick(time.delta());
        if wander.anim_timer.just_finished() && wander.direction.length_squared() > 0.01 {
            wander.anim_frame = animal_frame_index(wander.anim_frame + 1);
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                atlas.index = wander.anim_frame;
            }
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
