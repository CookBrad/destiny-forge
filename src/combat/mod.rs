mod attack;
mod block;
mod health;
mod hitbox;
mod player_block;
mod player_hurt;
mod plugin;
mod projectile;
mod weapon;

pub use attack::{
    animate_weapon_swing, resolve_weapon_hits, spawn_sheathed_sword, start_player_attack,
    sync_sheathed_weapon, tick_hit_flash, tick_player_attack, EnemyCorpse, PlayerAttack,
};
pub use block::{despawn_block_weapon, sync_block_weapon, update_player_block};
pub use player_block::PlayerBlock;
pub use player_hurt::{apply_player_hurt, tick_player_hit_flash, PlayerKnockback};
pub use projectile::{
    deflect_projectiles_with_swing, enemy_shoot_projectiles, move_enemy_projectiles,
    resolve_deflected_projectile_hits, resolve_enemy_projectiles, DeflectedProjectile,
    EnemyProjectile, ProjectileLifetime, ProjectileVelocity,
};
pub use health::{
    apply_enemy_contact_damage, damage_amount, health_bar_color, ContactDamageCooldown, Health,
    PLAYER_MAX_HEALTH,
};
pub use plugin::CombatPlugin;
pub use weapon::{EquippedWeapon, WeaponKind};