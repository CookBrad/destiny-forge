use bevy::prelude::*;

use crate::core::GameState;
use crate::graphics::{
    pixel_sprite, sprite_transform, DungeonSprite, GameSprites, PixelSheet, PIXEL_SCALE,
};
use crate::player::{DungeonPlayer, Facing, PlayerLoadout};

use super::{AttackCooldown, AttackHitbox, Health, Hurtbox};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_attack_input,
                tick_attack_cooldowns,
                move_attack_hitboxes,
                apply_hitbox_damage,
            )
                .chain()
                .run_if(in_state(GameState::Dungeon)),
        );
    }
}

fn player_attack_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (Entity, &Transform, &Facing, &mut AttackCooldown),
        With<DungeonPlayer>,
    >,
    loadout: Res<PlayerLoadout>,
    sprites: Res<GameSprites>,
) {
    if !keyboard.just_pressed(KeyCode::KeyJ) {
        return;
    }

    let Ok((entity, transform, facing, mut cooldown)) = player_query.get_single_mut() else {
        return;
    };

    if !cooldown.timer.finished() {
        return;
    }

    cooldown.timer.reset();

    let offset = match facing {
        Facing::Right => Vec2::new(loadout.weapon_reach() * 0.5, 0.0),
        Facing::Left => Vec2::new(-loadout.weapon_reach() * 0.5, 0.0),
    };

    let hitbox_size = Vec2::new(loadout.weapon_reach(), 28.0);
    let position = transform.translation.truncate() + offset;

    let mut slash_sprite =
        pixel_sprite(&sprites, PixelSheet::Dungeon, DungeonSprite::Slash.atlas_index());
    slash_sprite.custom_size = Some(hitbox_size / PIXEL_SCALE);
    slash_sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.85);

    commands.spawn((
        slash_sprite,
        sprite_transform(position.extend(20.0)),
        AttackHitbox {
            damage: loadout.weapon_damage(),
            lifetime: Timer::from_seconds(0.12, TimerMode::Once),
            facing: *facing,
            already_hit: vec![entity],
        },
        Name::new("PlayerAttackHitbox"),
    ));
}

fn tick_attack_cooldowns(time: Res<Time>, mut query: Query<&mut AttackCooldown>) {
    for mut cooldown in &mut query {
        cooldown.timer.tick(time.delta());
    }
}

fn move_attack_hitboxes(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AttackHitbox)>,
) {
    for (entity, mut hitbox) in &mut query {
        hitbox.lifetime.tick(time.delta());
        if hitbox.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn apply_hitbox_damage(
    mut hitbox_query: Query<(Entity, &mut AttackHitbox, &Transform)>,
    mut hurtbox_query: Query<(Entity, &Transform, &mut Health), With<Hurtbox>>,
) {
    for (_hitbox_entity, mut hitbox, hitbox_transform) in &mut hitbox_query {
        for (target_entity, target_transform, mut health) in &mut hurtbox_query {
            if hitbox.already_hit.contains(&target_entity) || !health.is_alive() {
                continue;
            }

            let distance = hitbox_transform
                .translation
                .truncate()
                .distance(target_transform.translation.truncate());

            if distance < 40.0 {
                health.take_damage(hitbox.damage);
                hitbox.already_hit.push(target_entity);
            }
        }
    }
}