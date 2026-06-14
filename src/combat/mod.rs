mod health;
mod hitbox;
mod hurtbox;
mod plugin;

pub use health::Health;
pub use hitbox::{AttackCooldown, AttackHitbox};
pub use hurtbox::Hurtbox;
pub use plugin::CombatPlugin;