mod attack;
mod block;
mod health;
mod hits;
mod hitbox;
mod hit_stop;
mod player_block;
mod player_death;
mod player_hurt;
mod projectile;
mod skills;
mod special_moves;
mod weapon;

pub use attack::{
    animate_weapon_swing, resolve_weapon_hits, spawn_sheathed_sword, start_player_attack,
    sync_sheathed_weapon, tick_hit_flash, tick_player_attack, EnemyCorpse, PlayerAttack,
};
pub use block::{despawn_block_weapon, sync_block_weapon, update_player_block};
pub use hit_stop::{tick_hit_stop, HitStop};
pub use player_block::PlayerBlock;
pub use special_moves::{
    animate_special_weapon, cleanup_special_weapon, resolve_special_move_hits,
    special_blocks_movement, special_move_speed, start_player_special_moves,
    tick_player_special_moves, tick_special_cooldowns, PlayerSpecialMove, SpecialCooldownState,
    SpecialMoveKind,
};
pub use player_death::{
    animate_player_death, detect_player_death, finish_player_death, hide_death_weapons,
    tick_player_death, PlayerDeath, PlayerFallDeath,
};
pub use player_hurt::{
    apply_player_hurt, tick_player_hit_flash, PlayerHitFlash, PlayerKnockback,
};
pub use projectile::{
    deflect_projectiles_with_swing, enemy_shoot_projectiles, move_enemy_projectiles,
    resolve_deflected_projectile_hits, resolve_enemy_projectiles, DeflectedProjectile,
    EnemyProjectile, ProjectileLifetime, ProjectileVelocity,
};
pub use health::{
    apply_enemy_contact_damage, damage_amount, health_bar_color, ContactDamageCooldown, Health,
    PLAYER_MAX_HEALTH,
};
pub use skills::{SkillBindings, SkillIconAssets, SkillKind, SKILL_SLOT_COUNT};
pub use weapon::{EquippedWeapon, WeaponKind};
