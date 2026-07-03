use bevy::prelude::*;

use super::player_hurt::apply_player_hurt;

pub const PLAYER_MAX_HEALTH: f32 = 100.0;

#[derive(Component, Clone, Copy, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn fraction(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }
}

/// Green at high health, yellow mid, red when low.
pub fn health_bar_color(ratio: f32) -> Color {
    let ratio = ratio.clamp(0.0, 1.0);
    const GREEN: Vec3 = Vec3::new(0.25, 0.9, 0.35);
    const YELLOW: Vec3 = Vec3::new(0.95, 0.88, 0.15);
    const RED: Vec3 = Vec3::new(0.92, 0.22, 0.2);

    let rgb = if ratio > 0.5 {
        GREEN.lerp(YELLOW, (1.0 - ratio) * 2.0)
    } else {
        YELLOW.lerp(RED, (0.5 - ratio) * 2.0)
    };

    Color::srgb(rgb.x, rgb.y, rgb.z)
}

pub fn damage_amount(attack_power: f32, defense: f32) -> f32 {
    (attack_power - defense).max(1.0)
}

const CONTACT_DAMAGE_INTERVAL: f32 = 0.75;

#[derive(Component)]
pub struct ContactDamageCooldown(pub Timer);

impl Default for ContactDamageCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(0.0, TimerMode::Once))
    }
}

pub fn apply_enemy_contact_damage(
    time: Res<Time>,
    mut commands: Commands,
    mut player: Query<
        (Entity, &Transform, &mut Health, &mut ContactDamageCooldown),
        With<crate::dungeon::DungeonPlayer>,
    >,
    enemies: Query<
        (
            &Transform,
            &crate::dungeon::EnemyHitbox,
            Option<&crate::dungeon::EnemyContactDamage>,
        ),
        (
            With<Health>,
            Without<crate::dungeon::DungeonPlayer>,
            Without<crate::combat::EnemyCorpse>,
        ),
    >,
) {
    let Ok((entity, player_transform, mut health, mut cooldown)) = player.get_single_mut() else {
        return;
    };

    if health.is_dead() {
        return;
    }

    let player_center = player_transform.translation.truncate();
    let player_half = crate::dungeon::player_half_extents();

    let mut contact_damage = 0.0_f32;
    let mut hurt_source = player_center;
    let mut closest_dist_sq = f32::MAX;
    let mut touching = false;

    for (transform, hitbox, damage) in &enemies {
        let enemy_center = transform.translation.truncate();
        let half = hitbox.0;
        let overlaps = (player_center.x - player_half.x) < (enemy_center.x + half.x)
            && (player_center.x + player_half.x) > (enemy_center.x - half.x)
            && (player_center.y - player_half.y) < (enemy_center.y + half.y)
            && (player_center.y + player_half.y) > (enemy_center.y - half.y);

        if overlaps {
            touching = true;
            contact_damage = contact_damage.max(damage.map(|d| d.0).unwrap_or(8.0));
            let dist_sq = player_center.distance_squared(enemy_center);
            if dist_sq < closest_dist_sq {
                closest_dist_sq = dist_sq;
                hurt_source = enemy_center;
            }
        }
    }

    if touching {
        cooldown.0.tick(time.delta());
        if cooldown.0.finished() {
            health.take_damage(contact_damage);
            apply_player_hurt(&mut commands, entity, player_transform, hurt_source, 1.0);
            cooldown.0 = Timer::from_seconds(CONTACT_DAMAGE_INTERVAL, TimerMode::Once);
        }
    } else {
        cooldown.0 = Timer::from_seconds(0.0, TimerMode::Once);
    }
}