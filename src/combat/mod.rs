use bevy::prelude::*;
use std::time::Duration;

use crate::enemy::{Enemy, Health};
use crate::player::{Direction, Player};

#[derive(Component)]
pub struct AttackHitbox {
    pub damage: f32,
    pub lifetime: Timer,
    pub _direction: Direction,
}

#[derive(Component)]
pub struct Attacking {
    pub timer: Timer,
}

impl Default for Attacking {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(0.3), TimerMode::Once),
        }
    }
}

pub fn handle_sword_attack(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    player_query: Query<(&Transform, &Player, Entity), With<Player>>,
    sprite_sheet: Res<crate::SpriteSheetLayout>,
    _inventory_query: Query<&crate::player::Inventory>,
    _selected_slot: Res<crate::inventory_ui::SelectedSlot>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let Ok((player_transform, player, player_entity)) = player_query.get_single() else {
            return;
        };

        // For now, allow attacking without weapon (fists)
        // Later you can check if selected slot has a weapon
        let damage = 1.0;

        // Calculate attack position based on player direction
        let attack_offset = match player.direction {
            Direction::Up => Vec2::new(0.0, 30.0),
            Direction::Down => Vec2::new(0.0, -30.0),
            Direction::Left => Vec2::new(-30.0, 0.0),
            Direction::Right => Vec2::new(30.0, 0.0),
        };

        let attack_pos = player_transform.translation.truncate() + attack_offset;

        // Spawn attack hitbox
        commands.spawn((
            Sprite {
                image: sprite_sheet.crops_texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: sprite_sheet.crops_layout.clone(),
                    index: 0, // Use a visual indicator for the attack
                }),
                ..Default::default()
            },
            Transform::from_translation(attack_pos.extend(1000.0)).with_scale(Vec3::splat(2.0)),
            AttackHitbox {
                damage,
                lifetime: Timer::new(Duration::from_secs_f32(0.2), TimerMode::Once),
                _direction: player.direction,
            },
            Name::new("AttackHitbox"),
        ));

        // Mark player as attacking
        commands.entity(player_entity).insert(Attacking::default());
    }
}

pub fn update_attack_hitboxes(
    time: Res<Time>,
    mut commands: Commands,
    mut hitbox_query: Query<(Entity, &mut AttackHitbox, &Transform)>,
    mut enemy_query: Query<(Entity, &mut Health, &Transform), (With<Enemy>, Without<AttackHitbox>)>,
) {
    for (hitbox_entity, mut hitbox, hitbox_transform) in hitbox_query.iter_mut() {
        hitbox.lifetime.tick(time.delta());

        // Check for collisions with enemies
        for (_enemy_entity, mut enemy_health, enemy_transform) in enemy_query.iter_mut() {
            if enemy_health.is_dead() {
                continue;
            }

            let distance = hitbox_transform
                .translation
                .truncate()
                .distance(enemy_transform.translation.truncate());

            if distance < 40.0 {
                // Hit enemy
                enemy_health.take_damage(hitbox.damage);
                println!(
                    "Enemy hit! Health: {}/{}",
                    enemy_health.current, enemy_health.max
                );
            }
        }

        // Despawn hitbox when lifetime expires
        if hitbox.lifetime.finished() {
            commands.entity(hitbox_entity).despawn();
        }
    }
}

pub fn enemy_attack_player(
    time: Res<Time>,
    mut player_query: Query<&mut Health, With<Player>>,
    enemy_query: Query<(&Transform, &Enemy), (With<Enemy>, Without<Player>)>,
    player_transform_query: Query<&Transform, With<Player>>,
) {
    let Ok(mut player_health) = player_query.get_single_mut() else {
        return;
    };

    let Ok(player_transform) = player_transform_query.get_single() else {
        return;
    };

    for (enemy_transform, enemy) in enemy_query.iter() {
        if !enemy.attack_cooldown.just_finished() {
            continue;
        }

        let distance = enemy_transform
            .translation
            .truncate()
            .distance(player_transform.translation.truncate());

        if distance <= enemy.attack_range {
            player_health.take_damage(enemy.attack_damage);
            println!(
                "Player hit by enemy! Health: {}/{}",
                player_health.current, player_health.max
            );
        }
    }
}

pub fn update_attacking_state(
    time: Res<Time>,
    mut commands: Commands,
    mut attacking_query: Query<(Entity, &mut Attacking)>,
) {
    for (entity, mut attacking) in attacking_query.iter_mut() {
        attacking.timer.tick(time.delta());
        if attacking.timer.finished() {
            commands.entity(entity).remove::<Attacking>();
        }
    }
}
