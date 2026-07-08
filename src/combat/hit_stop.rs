use bevy::prelude::*;

/// Brief freeze after solid hits so combat reads punchier.
#[derive(Resource, Default)]
pub struct HitStop {
    pub remaining: f32,
}

impl HitStop {
    pub fn request(&mut self, secs: f32) {
        self.remaining = self.remaining.max(secs);
    }

    pub fn is_active(&self) -> bool {
        self.remaining > 0.0
    }
}

pub const HIT_STOP_LIGHT: f32 = 0.035;
pub const HIT_STOP_HEAVY: f32 = 0.055;

pub fn tick_hit_stop(time: Res<Time>, mut hit_stop: ResMut<HitStop>) {
    if hit_stop.remaining > 0.0 {
        hit_stop.remaining = (hit_stop.remaining - time.delta_secs()).max(0.0);
    }
}
