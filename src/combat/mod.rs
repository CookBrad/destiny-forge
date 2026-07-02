mod attack;
mod health;
mod plugin;
mod weapon;

pub use attack::{
    animate_weapon_swing, resolve_weapon_hits, start_player_attack, tick_hit_flash,
    tick_player_attack, EnemyCorpse, PlayerAttack,
};
pub use health::{damage_amount, Health};
pub use plugin::CombatPlugin;
pub use weapon::{EquippedWeapon, WeaponKind};