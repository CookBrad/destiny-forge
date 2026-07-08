use bevy::prelude::*;

/// Hold block skill to raise the guard. Perfect-parry window opens on press.
#[derive(Component)]
pub struct PlayerBlock {
    pub active: bool,
    /// Counts down from press; while remaining > 0 and active, parry is perfect.
    pub parry_timer: Timer,
}

impl Default for PlayerBlock {
    fn default() -> Self {
        Self {
            active: false,
            parry_timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

/// Perfect-parry window length (seconds) after block is pressed.
pub const PARRY_WINDOW_SECS: f32 = 0.14;

impl PlayerBlock {
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// True during the short window after block press — full deflect / nullify.
    pub fn in_parry_window(&self) -> bool {
        self.active && !self.parry_timer.finished() && self.parry_timer.duration().as_secs_f32() > 0.0
    }

    pub fn begin_parry_window(&mut self) {
        self.parry_timer = Timer::from_seconds(PARRY_WINDOW_SECS, TimerMode::Once);
        self.parry_timer.reset();
    }

    pub fn tick_parry(&mut self, delta: std::time::Duration) {
        if self.active && !self.parry_timer.finished() {
            self.parry_timer.tick(delta);
        }
    }
}
