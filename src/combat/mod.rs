mod attack;
mod health;
mod plugin;
mod weapon;

pub use attack::{
    animate_weapon_swing, resolve_weapon_hits, start_player_attack, tick_hit_flash,
    tick_player_attack, EnemyCorpse, PlayerAttack,
};
pub use health::{
    apply_enemy_contact_damage, damage_amount, health_bar_color, ContactDamageCooldown, Health,
    PLAYER_MAX_HEALTH,
};
pub use plugin::CombatPlugin;
pub use weapon::{EquippedWeapon, WeaponKind};