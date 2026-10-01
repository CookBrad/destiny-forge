use std::collections::HashSet;

use bevy::prelude::*;

use crate::combat::{health_bar_color, EnemyCorpse, Health};
use crate::dungeon::{DungeonPlayer, EnemyHitbox, SpriteCanvas};
use crate::graphics::solid_white_pixel;

const ENEMY_BAR_HEIGHT: f32 = 4.0;
const ENEMY_BAR_PADDING: f32 = 4.0;
const ENEMY_BAR_Z: f32 = 8.0;

const PLAYER_BAR_WIDTH: f32 = 14.0;
const PLAYER_BAR_HEIGHT: f32 = 120.0;
const PLAYER_BAR_BORDER: f32 = 2.0;

#[derive(Resource)]
pub struct HealthBarAssets {
    pub pixel: Handle<Image>,
}

impl Default for HealthBarAssets {
    fn default() -> Self {
        Self {
            pixel: Handle::default(),
        }
    }
}

#[derive(Component)]
pub struct PlayerHealthBar;

#[derive(Component)]
pub(crate) struct PlayerHealthBarFill;

#[derive(Component)]
pub(crate) struct PlayerHealthBarBackground;

#[derive(Component)]
pub struct EnemyHealthBar {
    owner: Entity,
}

#[derive(Component)]
pub(crate) struct EnemyHealthBarBackground;

#[derive(Component)]
pub(crate) struct EnemyHealthBarFill;

pub fn setup_health_bar_assets(
    mut images: ResMut<Assets<Image>>,
    mut assets: ResMut<HealthBarAssets>,
) {
    if assets.pixel != Handle::default() {
        return;
    }

    assets.pixel = solid_white_pixel(&mut images);
}

pub fn spawn_player_health_bar(mut commands: Commands) {
    commands
        .spawn((
            PlayerHealthBar,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(72.0),
                width: Val::Px(PLAYER_BAR_WIDTH),
                height: Val::Px(PLAYER_BAR_HEIGHT),
                border: UiRect::all(Val::Px(PLAYER_BAR_BORDER)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.85)),
        ))
        .with_children(|bar| {
            bar.spawn((
                PlayerHealthBarBackground,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.18, 0.9)),
            ))
            .with_children(|track| {
                track.spawn((
                    PlayerHealthBarFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(health_bar_color(1.0)),
                ));
            });
        });
}

pub fn spawn_enemy_health_bars(
    mut commands: Commands,
    assets: Res<HealthBarAssets>,
    enemies: Query<
        (Entity, &Transform, &EnemyHitbox, &SpriteCanvas),
        (With<Health>, Without<DungeonPlayer>),
    >,
    existing: Query<&EnemyHealthBar>,
) {
    let covered: HashSet<Entity> = existing.iter().map(|bar| bar.owner).collect();

    for (owner, transform, hitbox, canvas) in &enemies {
        if covered.contains(&owner) {
            continue;
        }
        let offset_y = bar_offset_y(canvas);
        let bar_width = enemy_bar_width(hitbox);
        let bar_translation = transform.translation + Vec3::new(0.0, offset_y, ENEMY_BAR_Z);

        commands
            .spawn((
                EnemyHealthBar { owner },
                Sprite {
                    image: assets.pixel.clone(),
                    color: Color::srgba(0.08, 0.08, 0.1, 0.85),
                    custom_size: Some(Vec2::new(bar_width, ENEMY_BAR_HEIGHT)),
                    ..default()
                },
                Transform::from_translation(bar_translation),
                EnemyHealthBarBackground,
            ))
            .with_children(|bar| {
                bar.spawn((
                    EnemyHealthBarFill,
                    Sprite {
                        image: assets.pixel.clone(),
                        color: health_bar_color(1.0),
                        custom_size: Some(Vec2::new(bar_width, ENEMY_BAR_HEIGHT)),
                        ..default()
                    },
                    Transform::from_xyz(-bar_width * 0.5, 0.0, 0.01),
                ));
            });
    }
}

pub fn update_player_health_bar(
    player: Query<&Health, With<DungeonPlayer>>,
    mut fill: Query<(&mut Node, &mut BackgroundColor), With<PlayerHealthBarFill>>,
) {
    let Ok(health) = player.get_single() else {
        return;
    };

    let ratio = health.fraction();
    for (mut node, mut color) in &mut fill {
        node.height = Val::Percent(ratio * 100.0);
        color.0 = health_bar_color(ratio);
    }
}

pub fn despawn_orphan_enemy_health_bars(
    mut commands: Commands,
    owners: Query<Entity, With<Health>>,
    bars: Query<(Entity, &EnemyHealthBar)>,
) {
    for (bar_entity, bar) in &bars {
        if owners.get(bar.owner).is_err() {
            commands.entity(bar_entity).try_despawn_recursive();
        }
    }
}

pub fn update_enemy_health_bars(
    owners: Query<
        (
            Entity,
            &Transform,
            &Health,
            &EnemyHitbox,
            &SpriteCanvas,
            Option<&EnemyCorpse>,
        ),
        (
            Without<EnemyHealthBar>,
            Without<EnemyHealthBarFill>,
            Without<EnemyHealthBarBackground>,
        ),
    >,
    mut bars: Query<
        (Entity, &EnemyHealthBar, &mut Transform, &mut Visibility),
        (
            With<EnemyHealthBar>,
            Without<EnemyHealthBarFill>,
            Without<Health>,
        ),
    >,
    mut fills: Query<
        (&mut Sprite, &mut Transform),
        (
            With<EnemyHealthBarFill>,
            Without<EnemyHealthBar>,
            Without<EnemyHealthBarBackground>,
            Without<Health>,
        ),
    >,
    children: Query<&Children>,
) {
    for (bar_entity, bar, mut bar_transform, mut visibility) in &mut bars {
        let Ok((_, transform, health, hitbox, canvas, corpse)) = owners.get(bar.owner) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let ratio = health.fraction();
        let show = corpse.is_none() && ratio > 0.0;
        *visibility = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let bar_width = enemy_bar_width(hitbox);
        bar_transform.translation =
            transform.translation + Vec3::new(0.0, bar_offset_y(canvas), ENEMY_BAR_Z);

        let Ok(bar_children) = children.get(bar_entity) else {
            continue;
        };

        for child in bar_children.iter() {
            let Ok((mut fill_sprite, mut fill_transform)) = fills.get_mut(*child) else {
                continue;
            };

            fill_sprite.custom_size = Some(Vec2::new(bar_width * ratio, ENEMY_BAR_HEIGHT));
            fill_sprite.color = health_bar_color(ratio);
            fill_transform.translation.x = -bar_width * 0.5 + (bar_width * ratio) * 0.5;
        }
    }
}

pub fn cleanup_health_bars(
    mut commands: Commands,
    player_bars: Query<Entity, With<PlayerHealthBar>>,
    enemy_bars: Query<Entity, With<EnemyHealthBar>>,
) {
    for entity in player_bars.iter().chain(enemy_bars.iter()) {
        commands.entity(entity).try_despawn_recursive();
    }
}

/// Width stays the gameplay body. The sheet is taller, so Y uses the painted top.
fn enemy_bar_width(hitbox: &EnemyHitbox) -> f32 {
    hitbox.0.x * 2.0
}

fn bar_offset_y(canvas: &SpriteCanvas) -> f32 {
    canvas.top_above_origin() + ENEMY_BAR_PADDING
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_sits_above_the_painted_sheet() {
        let slime = SpriteCanvas::grounded(Vec2::new(64.0, 64.0), 32.0);
        assert_eq!(bar_offset_y(&slime), 48.0 + ENEMY_BAR_PADDING);
        let king = SpriteCanvas::grounded(Vec2::new(192.0, 192.0), 64.0);
        assert_eq!(bar_offset_y(&king), 160.0 + ENEMY_BAR_PADDING);
        assert_eq!(enemy_bar_width(&EnemyHitbox::standard()), 32.0);
        assert_eq!(
            enemy_bar_width(&EnemyHitbox::scaled(2.0)),
            64.0
        );
    }
}
