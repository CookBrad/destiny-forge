use bevy::prelude::*;

use crate::audio::CombatSfx;
use crate::player::Loadout;

use super::player_hurt::{apply_player_hurt, PlayerHitFlash};

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

pub(crate) type ContactDamagePlayer<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static mut Health,
        &'static mut ContactDamageCooldown,
        &'static super::player_block::PlayerBlock,
    ),
    (With<crate::dungeon::DungeonPlayer>, Without<PlayerHitFlash>),
>;

pub(crate) type ContactDamageEnemies<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        &'static crate::dungeon::EnemyHitbox,
        Option<&'static crate::dungeon::EnemyContactDamage>,
    ),
    (
        With<Health>,
        Without<crate::dungeon::DungeonPlayer>,
        Without<crate::combat::EnemyCorpse>,
    ),
>;

pub fn apply_enemy_contact_damage(
    time: Res<Time>,
    loadout: Res<Loadout>,
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    mut player: ContactDamagePlayer,
    enemies: ContactDamageEnemies,
) {
    let Ok((entity, player_transform, mut health, mut cooldown, block)) = player.get_single_mut()
    else {
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
            // Perfect parry nullifies one contact hit.
            if block.in_parry_window() {
                sfx.send(CombatSfx::Parry);
                cooldown.0 = Timer::from_seconds(CONTACT_DAMAGE_INTERVAL, TimerMode::Once);
                return;
            }

            // Guard reduces contact damage while held.
            let defense_bonus = if block.is_active() {
                loadout.total_defense() + contact_damage * 0.5
            } else {
                loadout.total_defense()
            };

            health.take_damage(damage_amount(contact_damage, defense_bonus));
            apply_player_hurt(
                &mut commands,
                entity,
                player_transform,
                hurt_source,
                if block.is_active() { 0.45 } else { 1.0 },
                loadout.knockback_resist(),
            );
            sfx.send(if block.is_active() {
                CombatSfx::Block
            } else {
                CombatSfx::EnemyMelee
            });
            cooldown.0 = Timer::from_seconds(CONTACT_DAMAGE_INTERVAL, TimerMode::Once);
        }
    } else {
        cooldown.0 = Timer::from_seconds(0.0, TimerMode::Once);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{player_half_extents, DungeonPlayer, EnemyContactDamage, EnemyHitbox};
    use crate::graphics::TILE;
    use crate::player::Loadout;

    #[test]
    fn an_overlapping_enemy_deals_contact_damage() {
        let mut app = contact_app();
        let power = EnemyContactDamage(crate::dungeon::EnemyKind::Skeleton.projectile_damage());
        let dealt = damage_amount(power.0, Loadout::default().total_defense());
        let starting = dealt + dealt;
        app.world_mut().spawn(hunter(starting));
        app.world_mut().spawn((
            Transform::default(),
            EnemyHitbox(Vec2::splat(TILE * 0.5)),
            Health::new(power.0),
            power,
        ));
        app.add_systems(Update, apply_enemy_contact_damage);
        app.update();
        let health = app
            .world_mut()
            .query_filtered::<&Health, With<DungeonPlayer>>()
            .single(app.world())
            .current;
        let flashed = app
            .world_mut()
            .query::<&PlayerHitFlash>()
            .iter(app.world())
            .len();
        assert_eq!(health, starting - dealt);
        assert_eq!(flashed, 1);
    }

    fn contact_app() -> App {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        app.insert_resource(Loadout::default());
        app.add_event::<CombatSfx>();
        app
    }

    fn hunter(health: f32) -> impl Bundle {
        (
            DungeonPlayer,
            Transform::default(),
            Health::new(health),
            ContactDamageCooldown::default(),
            super::super::player_block::PlayerBlock::default(),
        )
    }

    #[test]
    fn an_enemy_a_tile_past_the_hunter_does_not_deal_contact_damage() {
        let mut app = contact_app();
        let power = crate::dungeon::EnemyKind::Skeleton.projectile_damage();
        let starting = power + power;
        app.world_mut().spawn(hunter(starting));
        app.world_mut().spawn((
            Transform::from_xyz(player_half_extents().x + TILE, 0.0, 0.0),
            EnemyHitbox(Vec2::splat(TILE * 0.5)),
            Health::new(power),
            EnemyContactDamage(power),
        ));
        app.add_systems(Update, apply_enemy_contact_damage);
        app.update();
        let health = app
            .world_mut()
            .query_filtered::<&Health, With<DungeonPlayer>>()
            .single(app.world())
            .current;
        let flashed = app
            .world_mut()
            .query::<&PlayerHitFlash>()
            .iter(app.world())
            .len();
        assert_eq!(health, starting);
        assert_eq!(flashed, 0);
    }

    #[test]
    fn a_dead_hunter_does_not_take_contact_damage() {
        let mut app = contact_app();
        let power = crate::dungeon::EnemyKind::Skeleton.projectile_damage();
        app.world_mut().spawn(hunter(0.0));
        app.world_mut().spawn((
            Transform::default(),
            EnemyHitbox(Vec2::splat(TILE * 0.5)),
            Health::new(power),
            EnemyContactDamage(power),
        ));
        app.add_systems(Update, apply_enemy_contact_damage);
        app.update();
        let health = app
            .world_mut()
            .query_filtered::<&Health, With<DungeonPlayer>>()
            .single(app.world())
            .current;
        let flashed = app
            .world_mut()
            .query::<&PlayerHitFlash>()
            .iter(app.world())
            .len();
        assert_eq!(health, 0.0);
        assert_eq!(flashed, 0);
    }
}
