use bevy::prelude::*;

use crate::ui::eat_feedback::FoodEaten;

use super::movement::{OverworldPlayer, OverworldVelocity};
use super::sprites::OverworldArt;

pub const EAT_CYCLE_FRAME_SECS: f32 = 0.18;
pub const EAT_CYCLE_FRAME_ORDER: [usize; 6] = [0, 1, 2, 3, 2, 3];

#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct HomesteadEatCycle {
    pub elapsed_secs: f32,
}

impl HomesteadEatCycle {
    pub fn duration_secs() -> f32 {
        EAT_CYCLE_FRAME_SECS * EAT_CYCLE_FRAME_ORDER.len() as f32
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed_secs >= Self::duration_secs()
    }

    pub fn eat_frame(&self) -> usize {
        let step = (self.elapsed_secs / EAT_CYCLE_FRAME_SECS) as usize;
        EAT_CYCLE_FRAME_ORDER[step.min(EAT_CYCLE_FRAME_ORDER.len() - 1)]
    }
}

pub fn start_homestead_eat_cycle(
    mut commands: Commands,
    mut food_eaten: EventReader<FoodEaten>,
    mut players: Query<(Entity, &mut OverworldVelocity), With<OverworldPlayer>>,
) {
    if food_eaten.read().last().is_none() {
        return;
    }
    for (player, mut velocity) in &mut players {
        velocity.x = 0.0;
        velocity.y = 0.0;
        commands.entity(player).insert(HomesteadEatCycle::default());
    }
}

pub fn play_homestead_eat_cycle(
    real_time: Res<Time<Real>>,
    art: Res<OverworldArt>,
    mut commands: Commands,
    mut eaters: Query<(Entity, &mut HomesteadEatCycle, &mut Sprite), With<OverworldPlayer>>,
) {
    for (player, mut cycle, mut sprite) in &mut eaters {
        cycle.elapsed_secs += real_time.delta_secs();
        sprite.rect = None;
        if cycle.is_finished() {
            sprite.image = art.player.frame_handle(false, 0);
            commands.entity(player).remove::<HomesteadEatCycle>();
            continue;
        }
        sprite.image = art.player.eat_frame_handle(cycle.eat_frame());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::MaterialId;
    use bevy::ecs::system::RunSystemOnce;

    fn app_with_homestead_art() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Image>()
            .init_asset::<TextureAtlasLayout>()
            .add_event::<FoodEaten>();
        let art = app.world_mut().resource_scope(
            |world, mut layouts: Mut<Assets<TextureAtlasLayout>>| {
                world.resource_scope(|world, mut images: Mut<Assets<Image>>| {
                    OverworldArt::load(world.resource::<AssetServer>(), &mut layouts, &mut images)
                })
            },
        );
        app.insert_resource(art);
        app
    }

    fn spawn_idle_player(app: &mut App) -> Entity {
        let idle = app
            .world()
            .resource::<OverworldArt>()
            .player
            .frame_handle(false, 0);
        app.world_mut()
            .spawn((
                OverworldPlayer,
                OverworldVelocity::default(),
                Sprite {
                    image: idle,
                    ..default()
                },
            ))
            .id()
    }

    #[test]
    fn eat_cycle_reaches_lifts_bites_and_chews_then_ends() {
        let frames: Vec<usize> = (0..EAT_CYCLE_FRAME_ORDER.len())
            .map(|step| {
                HomesteadEatCycle {
                    elapsed_secs: (step as f32 + 0.5) * EAT_CYCLE_FRAME_SECS,
                }
                .eat_frame()
            })
            .collect();
        assert_eq!(frames, vec![0, 1, 2, 3, 2, 3]);
        assert!(HomesteadEatCycle::duration_secs() >= 1.0);
        assert!(!HomesteadEatCycle::default().is_finished());
        assert!(HomesteadEatCycle {
            elapsed_secs: HomesteadEatCycle::duration_secs()
        }
        .is_finished());
    }

    #[test]
    fn eating_food_starts_the_eat_cycle_and_shows_eat_sprite_frames() {
        let mut app = app_with_homestead_art();
        let player = spawn_idle_player(&mut app);
        app.world_mut().send_event(FoodEaten {
            food: MaterialId::PotatoStew,
        });

        app.world_mut()
            .run_system_once(start_homestead_eat_cycle)
            .unwrap();
        app.world_mut().flush();
        assert_eq!(
            app.world().get::<HomesteadEatCycle>(player),
            Some(&HomesteadEatCycle::default())
        );

        app.world_mut()
            .run_system_once(play_homestead_eat_cycle)
            .unwrap();
        let reach_frame = app.world().resource::<OverworldArt>().player.eat[0].clone();
        assert_eq!(
            app.world().get::<Sprite>(player).unwrap().image,
            reach_frame
        );

        app.world_mut()
            .get_mut::<HomesteadEatCycle>(player)
            .unwrap()
            .elapsed_secs = EAT_CYCLE_FRAME_SECS * 2.5;
        app.world_mut()
            .run_system_once(play_homestead_eat_cycle)
            .unwrap();
        let bite_frame = app.world().resource::<OverworldArt>().player.eat[2].clone();
        assert_eq!(app.world().get::<Sprite>(player).unwrap().image, bite_frame);
    }

    #[test]
    fn finished_eat_cycle_returns_to_idle_sprite() {
        let mut app = app_with_homestead_art();
        let player = spawn_idle_player(&mut app);
        app.world_mut()
            .entity_mut(player)
            .insert(HomesteadEatCycle {
                elapsed_secs: HomesteadEatCycle::duration_secs(),
            });

        app.world_mut()
            .run_system_once(play_homestead_eat_cycle)
            .unwrap();
        app.world_mut().flush();

        let idle = app
            .world()
            .resource::<OverworldArt>()
            .player
            .frame_handle(false, 0);
        assert_eq!(app.world().get::<Sprite>(player).unwrap().image, idle);
        assert!(app.world().get::<HomesteadEatCycle>(player).is_none());
    }

    #[test]
    fn no_meal_means_no_eat_cycle() {
        let mut app = app_with_homestead_art();
        let player = spawn_idle_player(&mut app);
        app.world_mut()
            .run_system_once(start_homestead_eat_cycle)
            .unwrap();
        app.world_mut().flush();
        assert!(app.world().get::<HomesteadEatCycle>(player).is_none());
    }
}
