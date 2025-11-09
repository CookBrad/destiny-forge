use bevy::prelude::*;
use bevy::ui::BackgroundColor;
use std::time::Duration;

#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
    pub detection_range: f32,
    pub attack_range: f32,
    pub attack_cooldown: Timer,
    pub attack_damage: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            speed: 2.0,
            detection_range: 150.0,
            attack_range: 50.0,
            attack_cooldown: Timer::new(Duration::from_secs_f32(1.5), TimerMode::Repeating),
            attack_damage: 10.0,
        }
    }
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.current = (self.current - damage).max(0.0);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

#[derive(Component)]
pub struct EnemyHealthBar {
    pub enemy_entity: Entity,
}

pub fn spawn_enemy(
    commands: &mut Commands,
    position: Vec3,
    sprite_sheet: &crate::SpriteSheetLayout,
) -> Entity {
    let enemy_entity = commands
        .spawn((
            Sprite {
                image: sprite_sheet.player_texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: sprite_sheet.player_layout.clone(),
                    index: 0, // Use a different sprite index for enemies
                }),
                ..Default::default()
            },
            Transform::from_translation(position).with_scale(Vec3::splat(3.0)),
            Enemy::default(),
            Health::new(50.0),
            Name::new("Enemy"),
        ))
        .id();

    // Spawn health bar above enemy using UI nodes
    // Position will be updated by update_enemy_health_bars system
    // Spawn health bar background
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(30.0),
            height: Val::Px(4.0),
            left: Val::Px(0.0), // Will be updated by system
            top: Val::Px(0.0),  // Will be updated by system
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)), // Dark background
        EnemyHealthBar { enemy_entity },
        Name::new("EnemyHealthBarBackground"),
    ));

    // Spawn health bar fill (the actual health indicator)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(30.0),
            height: Val::Px(4.0),
            left: Val::Px(0.0), // Will be updated by system
            top: Val::Px(0.0),  // Will be updated by system
            ..default()
        },
        BackgroundColor(Color::srgb(1.0, 0.0, 0.0)), // Red health bar (will change based on health)
        EnemyHealthBar { enemy_entity },
        Name::new("EnemyHealthBarFill"),
    ));

    enemy_entity
}

pub fn enemy_ai(
    time: Res<Time>,
    mut enemy_query: Query<(&mut Transform, &mut Enemy, &Health), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    for (mut enemy_transform, mut enemy, health) in enemy_query.iter_mut() {
        if health.is_dead() {
            continue;
        }

        enemy.attack_cooldown.tick(time.delta());

        let distance = enemy_transform
            .translation
            .truncate()
            .distance(player_transform.translation.truncate());

        // Move towards player if in detection range
        if distance <= enemy.detection_range && distance > enemy.attack_range {
            let direction = (player_transform.translation.truncate()
                - enemy_transform.translation.truncate())
            .normalize_or_zero();
            enemy_transform.translation +=
                (direction * enemy.speed * time.delta_secs()).extend(0.0);
            enemy_transform.translation.z = 500.0 - enemy_transform.translation.y;
        }

        // Attack if in range
        if distance <= enemy.attack_range && enemy.attack_cooldown.just_finished() {
            // Attack player
            // This will be handled by the combat system
        }
    }
}

pub fn despawn_dead_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Health), With<Enemy>>,
    health_bar_query: Query<(Entity, &EnemyHealthBar)>,
) {
    for (entity, health) in query.iter() {
        if health.is_dead() {
            // Despawn health bars associated with this enemy
            for (health_bar_entity, health_bar) in health_bar_query.iter() {
                if health_bar.enemy_entity == entity {
                    commands.entity(health_bar_entity).despawn();
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_enemy_health_bars(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    enemy_query: Query<(Entity, &Transform, &Health), With<Enemy>>,
    mut health_bar_query: Query<
        (&EnemyHealthBar, &mut Node, &mut BackgroundColor, &Name),
        Without<Enemy>,
    >,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    for (enemy_entity, enemy_transform, enemy_health) in enemy_query.iter() {
        let health_percentage = enemy_health.current / enemy_health.max;
        let health_bar_offset = Vec3::new(0.0, 50.0, 0.0);
        let world_pos = enemy_transform.translation + health_bar_offset;

        // Convert world position to screen position
        let screen_pos = world_to_screen(window, camera, camera_transform, world_pos);

        for (health_bar, mut node, mut bg_color, name) in health_bar_query.iter_mut() {
            if health_bar.enemy_entity == enemy_entity {
                // Update position to follow enemy (UI coordinates)
                node.left = Val::Px(screen_pos.x - 15.0);
                node.top = Val::Px(screen_pos.y - 2.0);

                // Update health bar fill width and color based on health percentage
                if name.as_str() == "EnemyHealthBarFill" {
                    let base_width = 30.0;
                    let current_width = base_width * health_percentage;
                    node.width = Val::Px(current_width);

                    // Adjust position so the bar shrinks from the right
                    let width_diff = base_width - current_width;
                    node.left = Val::Px(screen_pos.x - 15.0 + width_diff / 2.0);

                    // Change color based on health: green -> yellow -> red
                    if health_percentage > 0.6 {
                        *bg_color = Color::srgb(0.0, 1.0, 0.0).into(); // Green
                    } else if health_percentage > 0.3 {
                        *bg_color = Color::srgb(1.0, 1.0, 0.0).into(); // Yellow
                    } else {
                        *bg_color = Color::srgb(1.0, 0.0, 0.0).into(); // Red
                    }
                }
            }
        }
    }
}

// Helper function to convert world position to screen position
fn world_to_screen(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    world_pos: Vec3,
) -> Vec2 {
    // Get the window size
    let window_size = Vec2::new(window.width(), window.height());

    // Convert world position to NDC (Normalized Device Coordinates)
    let ndc = camera.world_to_ndc(camera_transform, world_pos);

    if let Some(ndc) = ndc {
        // Convert NDC to screen coordinates
        // NDC: (-1, -1) bottom-left to (1, 1) top-right
        // Screen: (0, 0) top-left to (width, height) bottom-right
        let screen_x = (ndc.x + 1.0) / 2.0 * window_size.x;
        let screen_y = (1.0 - ndc.y) / 2.0 * window_size.y; // Invert Y for screen coordinates
        Vec2::new(screen_x, screen_y)
    } else {
        // If conversion fails, return a default position
        Vec2::new(0.0, 0.0)
    }
}

use crate::player::Player;
