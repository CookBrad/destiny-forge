use bevy::audio::{AudioPlayer, PlaybackSettings, Volume};
use bevy::prelude::*;

use super::settings::AudioSettings;

#[derive(Event, Clone, Copy, Debug)]
pub enum CombatSfx {
    SwordSwing,
    SwordHit,
    HeavyHit,
    Block,
    Parry,
    Charge,
    Spin,
    EnemyShoot,
    SlimeShoot,
    SlimeBurst,
    GroundSlam,
    EnemyMelee,
    PlayerHurt,
    BossCharge,
}

#[derive(Resource, Default)]
pub struct CombatSfxAssets {
    pub sword_swing: Handle<AudioSource>,
    pub sword_hit: Handle<AudioSource>,
    pub heavy_hit: Handle<AudioSource>,
    pub block: Handle<AudioSource>,
    pub parry: Handle<AudioSource>,
    pub charge: Handle<AudioSource>,
    pub spin: Handle<AudioSource>,
    pub enemy_shoot: Handle<AudioSource>,
    pub slime_shoot: Handle<AudioSource>,
    pub slime_burst: Handle<AudioSource>,
    pub ground_slam: Handle<AudioSource>,
    pub enemy_melee: Handle<AudioSource>,
    pub player_hurt: Handle<AudioSource>,
    pub boss_charge: Handle<AudioSource>,
    loaded: bool,
}

impl CombatSfxAssets {
    pub fn load(&mut self, server: &AssetServer) {
        if self.loaded {
            return;
        }

        self.sword_swing = server.load("audio/sfx/sword_swing.ogg");
        self.sword_hit = server.load("audio/sfx/sword_hit.ogg");
        self.heavy_hit = server.load("audio/sfx/heavy_hit.ogg");
        self.block = server.load("audio/sfx/block.ogg");
        self.parry = server.load("audio/sfx/parry.ogg");
        self.charge = server.load("audio/sfx/charge.ogg");
        self.spin = server.load("audio/sfx/spin.ogg");
        self.enemy_shoot = server.load("audio/sfx/enemy_shoot.ogg");
        self.slime_shoot = server.load("audio/sfx/slime_shoot.ogg");
        self.slime_burst = server.load("audio/sfx/slime_burst.ogg");
        self.ground_slam = server.load("audio/sfx/ground_slam.ogg");
        self.enemy_melee = server.load("audio/sfx/enemy_melee.ogg");
        self.player_hurt = server.load("audio/sfx/player_hurt.ogg");
        self.boss_charge = server.load("audio/sfx/boss_charge.ogg");
        self.loaded = true;
    }

    fn clip(&self, sfx: CombatSfx) -> (&Handle<AudioSource>, f32, f32) {
        match sfx {
            CombatSfx::SwordSwing => (&self.sword_swing, 0.36, 1.15),
            CombatSfx::SwordHit => (&self.sword_hit, 0.4, 1.1),
            CombatSfx::HeavyHit => (&self.heavy_hit, 0.64, 1.0),
            CombatSfx::Block => (&self.block, 0.52, 1.0),
            CombatSfx::Parry => (&self.parry, 0.55, 1.0),
            CombatSfx::Charge => (&self.charge, 0.52, 1.18),
            CombatSfx::Spin => (&self.spin, 0.46, 1.22),
            CombatSfx::EnemyShoot => (&self.enemy_shoot, 0.5, 1.0),
            CombatSfx::SlimeShoot => (&self.slime_shoot, 0.46, 1.0),
            CombatSfx::SlimeBurst => (&self.slime_burst, 0.44, 1.0),
            CombatSfx::GroundSlam => (&self.ground_slam, 0.58, 1.0),
            CombatSfx::EnemyMelee => (&self.enemy_melee, 0.5, 1.0),
            CombatSfx::PlayerHurt => (&self.player_hurt, 0.54, 1.0),
            CombatSfx::BossCharge => (&self.boss_charge, 0.52, 1.0),
        }
    }
}

pub fn setup_combat_sfx(mut assets: ResMut<CombatSfxAssets>, server: Res<AssetServer>) {
    assets.load(&server);
}

pub fn play_combat_sfx(
    mut commands: Commands,
    mut events: EventReader<CombatSfx>,
    assets: Res<CombatSfxAssets>,
    settings: Res<AudioSettings>,
) {
    let gain = settings.sfx_gain();
    if gain <= 0.0 {
        events.clear();
        return;
    }

    for event in events.read() {
        let (clip, volume, speed) = assets.clip(*event);
        commands.spawn((
            AudioPlayer::new(clip.clone()),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::new(volume * gain))
                .with_speed(speed),
        ));
    }
}