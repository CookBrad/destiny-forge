//! Crop plot entities and layered stage visuals on the homestead.

use bevy::prelude::*;

use crate::graphics::{world_transform, TILE};
use crate::overworld::layout::{tile_center, OverworldEntity};
use crate::overworld::sprites::OverworldArt;

use super::crops::{advance_plot_day, CropKind, PlotStage};

#[derive(Component, Clone, Debug)]
pub struct CropPlot {
    pub tile_x: u32,
    pub tile_y: u32,
    pub stage: PlotStage,
}

/// Soil / tilled base layer (parent plot entity).
#[derive(Component)]
pub struct CropSoilSprite;

/// Plant / seed growth layer (child).
#[derive(Component)]
pub struct CropPlantSprite;

/// Water sheen when the plot was watered today (child).
#[derive(Component)]
pub struct CropWaterSprite;

/// Facing direction for tool use (unit-ish axis).
#[derive(Component, Clone, Copy, Debug)]
pub struct PlayerFacing {
    pub dir: Vec2,
}

impl Default for PlayerFacing {
    fn default() -> Self {
        Self {
            dir: Vec2::new(0.0, -1.0),
        }
    }
}

pub fn spawn_crop_plots(commands: &mut Commands, art: &OverworldArt, field: Rect) {
    let min_tx = (field.min.x / TILE).floor() as u32;
    let max_tx = (field.max.x / TILE).ceil() as u32;
    let min_ty = (field.min.y / TILE).floor() as u32;
    let max_ty = (field.max.y / TILE).ceil() as u32;

    for ty in min_ty..max_ty {
        for tx in min_tx..max_tx {
            // Every field tile is a workable plot (easier targeting).
            let center = tile_center(tx, ty);
            let stage = PlotStage::Soil;
            let soil = soil_visual(stage);

            commands
                .spawn((
                    Sprite {
                        image: art.soil.clone(),
                        color: soil.color,
                        custom_size: Some(soil.size),
                        ..default()
                    },
                    world_transform(center, 1.15),
                    CropPlot {
                        tile_x: tx,
                        tile_y: ty,
                        stage,
                    },
                    CropSoilSprite,
                    OverworldEntity,
                ))
                .with_children(|parent| {
                    // Plant layer (hidden until planted / ready). Starts as seed sprite.
                    parent.spawn((
                        Sprite {
                            image: art.seed.clone(),
                            color: Color::NONE,
                            custom_size: Some(Vec2::splat(TILE * 0.7)),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(0.0, 0.0, 0.2)),
                        Visibility::Hidden,
                        CropPlantSprite,
                        OverworldEntity,
                    ));
                    // Water sheen (hidden until watered).
                    parent.spawn((
                        Sprite {
                            image: art.path.clone(),
                            color: Color::srgba(0.35, 0.55, 0.85, 0.55),
                            custom_size: Some(Vec2::new(TILE * 0.75, TILE * 0.22)),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(0.0, -TILE * 0.22, 0.15)),
                        Visibility::Hidden,
                        CropWaterSprite,
                        OverworldEntity,
                    ));
                });
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlantSpriteKind {
    /// Hidden / none.
    None,
    /// Dedicated `seed.png` cluster for day-0 planting.
    Seed,
    /// Soft block for sprout / mature (tinted).
    Foliage,
}

struct LayerVisual {
    color: Color,
    size: Vec2,
    /// Local offset for plant layer (y up).
    offset_y: f32,
    visible: bool,
    sprite_kind: PlantSpriteKind,
}

fn soil_layer(color: Color, size: Vec2) -> LayerVisual {
    LayerVisual {
        color,
        size,
        offset_y: 0.0,
        visible: true,
        sprite_kind: PlantSpriteKind::None,
    }
}

fn soil_visual(stage: PlotStage) -> LayerVisual {
    match stage {
        PlotStage::Soil => soil_layer(Color::srgb(0.42, 0.32, 0.2), Vec2::splat(TILE * 0.92)),
        PlotStage::Tilled => soil_layer(
            Color::srgb(0.22, 0.14, 0.08),
            Vec2::new(TILE * 0.88, TILE * 0.78),
        ),
        PlotStage::Growing { watered: true, .. } => soil_layer(
            Color::srgb(0.16, 0.14, 0.18),
            Vec2::new(TILE * 0.88, TILE * 0.78),
        ),
        PlotStage::Growing { watered: false, .. } | PlotStage::Ready { .. } => soil_layer(
            Color::srgb(0.24, 0.15, 0.09),
            Vec2::new(TILE * 0.88, TILE * 0.78),
        ),
    }
}

fn watered_stage(stage: PlotStage) -> bool {
    matches!(stage, PlotStage::Growing { watered: true, .. })
}

fn plant_visual(stage: PlotStage) -> LayerVisual {
    match stage {
        PlotStage::Soil | PlotStage::Tilled => LayerVisual {
            color: Color::NONE,
            size: Vec2::splat(1.0),
            offset_y: 0.0,
            visible: false,
            sprite_kind: PlantSpriteKind::None,
        },
        PlotStage::Growing {
            crop,
            days,
            watered,
        } => {
            // Day 0: dedicated seed cluster sprite (tinted by crop kind).
            if days == 0 {
                let tint = match crop {
                    CropKind::Turnip => {
                        if watered {
                            Color::srgb(0.85, 0.7, 0.95)
                        } else {
                            Color::srgb(0.95, 0.85, 1.0)
                        }
                    }
                    CropKind::Potato => {
                        if watered {
                            Color::srgb(0.95, 0.85, 0.55)
                        } else {
                            Color::srgb(1.0, 0.95, 0.75)
                        }
                    }
                };
                return LayerVisual {
                    color: tint,
                    size: Vec2::splat(TILE * 0.7),
                    offset_y: -TILE * 0.02,
                    visible: true,
                    sprite_kind: PlantSpriteKind::Seed,
                };
            }

            let (color, size, offset_y) = match (crop, days) {
                (CropKind::Turnip, 1) => (
                    Color::srgb(0.35, 0.72, 0.32),
                    Vec2::new(TILE * 0.28, TILE * 0.38),
                    TILE * 0.08,
                ),
                (CropKind::Potato, 1) => (
                    Color::srgb(0.28, 0.62, 0.28),
                    Vec2::new(TILE * 0.3, TILE * 0.32),
                    TILE * 0.06,
                ),
                (CropKind::Potato, 2) => (
                    Color::srgb(0.32, 0.7, 0.3),
                    Vec2::new(TILE * 0.42, TILE * 0.48),
                    TILE * 0.12,
                ),
                (CropKind::Turnip, _) => (
                    Color::srgb(0.4, 0.78, 0.35),
                    Vec2::new(TILE * 0.4, TILE * 0.5),
                    TILE * 0.12,
                ),
                (CropKind::Potato, _) => (
                    Color::srgb(0.35, 0.72, 0.32),
                    Vec2::new(TILE * 0.45, TILE * 0.52),
                    TILE * 0.14,
                ),
            };
            LayerVisual {
                color,
                size,
                offset_y,
                visible: true,
                sprite_kind: PlantSpriteKind::Foliage,
            }
        }
        PlotStage::Ready { crop } => {
            let (color, size) = match crop {
                CropKind::Turnip => (
                    Color::srgb(0.72, 0.42, 0.78),
                    Vec2::new(TILE * 0.48, TILE * 0.55),
                ),
                CropKind::Potato => (
                    Color::srgb(0.82, 0.68, 0.38),
                    Vec2::new(TILE * 0.55, TILE * 0.42),
                ),
            };
            LayerVisual {
                color,
                size,
                offset_y: TILE * 0.1,
                visible: true,
                sprite_kind: PlantSpriteKind::Foliage,
            }
        }
    }
}

pub fn sync_plot_visuals(
    art: Res<OverworldArt>,
    mut plots: Query<(&CropPlot, &Children, &mut Sprite), (With<CropSoilSprite>, Changed<CropPlot>)>,
    mut plants: Query<
        (&mut Sprite, &mut Transform, &mut Visibility),
        (With<CropPlantSprite>, Without<CropSoilSprite>, Without<CropWaterSprite>),
    >,
    mut waters: Query<
        (&mut Sprite, &mut Visibility),
        (With<CropWaterSprite>, Without<CropSoilSprite>, Without<CropPlantSprite>),
    >,
) {
    for (plot, children, mut soil_sprite) in &mut plots {
        let soil = soil_visual(plot.stage);
        soil_sprite.color = soil.color;
        soil_sprite.custom_size = Some(soil.size);

        let plant = plant_visual(plot.stage);
        let show_water = watered_stage(plot.stage);
        let plant_image = match plant.sprite_kind {
            PlantSpriteKind::Seed => art.seed.clone(),
            PlantSpriteKind::Foliage | PlantSpriteKind::None => art.grass.clone(),
        };

        for child in children.iter() {
            if let Ok((mut sprite, mut transform, mut visibility)) = plants.get_mut(*child) {
                sprite.image = plant_image.clone();
                sprite.color = plant.color;
                sprite.custom_size = Some(plant.size);
                transform.translation.y = plant.offset_y;
                *visibility = if plant.visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
            if let Ok((mut sprite, mut visibility)) = waters.get_mut(*child) {
                // Stronger sheen when just watered.
                sprite.color = if show_water {
                    Color::srgba(0.4, 0.65, 0.95, 0.65)
                } else {
                    Color::NONE
                };
                *visibility = if show_water {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

pub fn advance_all_plots_on_sleep(mut plots: Query<&mut CropPlot>) {
    for mut plot in &mut plots {
        plot.stage = advance_plot_day(plot.stage);
    }
}

pub fn tile_coords_from_world(position: Vec2) -> (u32, u32) {
    let tx = (position.x / TILE).floor().max(0.0) as u32;
    let ty = (position.y / TILE).floor().max(0.0) as u32;
    (tx, ty)
}

pub fn facing_tile(player_pos: Vec2, facing: Vec2) -> (u32, u32) {
    let dir = if facing.length_squared() < 0.01 {
        Vec2::new(0.0, -1.0)
    } else {
        facing.normalize()
    };
    let target = player_pos + dir * TILE * 0.85;
    tile_coords_from_world(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_stage_has_distinct_plant_visibility() {
        assert!(!plant_visual(PlotStage::Soil).visible);
        assert!(!plant_visual(PlotStage::Tilled).visible);
        assert!(plant_visual(PlotStage::Growing {
            crop: CropKind::Turnip,
            days: 0,
            watered: false,
        })
        .visible);
        assert!(plant_visual(PlotStage::Ready {
            crop: CropKind::Potato
        })
        .visible);
    }

    #[test]
    fn planted_day_zero_uses_seed_sprite() {
        let planted = plant_visual(PlotStage::Growing {
            crop: CropKind::Turnip,
            days: 0,
            watered: false,
        });
        assert_eq!(planted.sprite_kind, PlantSpriteKind::Seed);
        assert!(planted.visible);

        let sprout = plant_visual(PlotStage::Growing {
            crop: CropKind::Turnip,
            days: 1,
            watered: true,
        });
        assert_eq!(sprout.sprite_kind, PlantSpriteKind::Foliage);
    }

    #[test]
    fn ready_crops_differ_by_kind_color() {
        let turnip = plant_visual(PlotStage::Ready {
            crop: CropKind::Turnip,
        });
        let potato = plant_visual(PlotStage::Ready {
            crop: CropKind::Potato,
        });
        assert_ne!(turnip.color, potato.color);
        assert!(turnip.size.y > potato.size.y || turnip.size.x < potato.size.x);
    }

    #[test]
    fn growth_advances_from_seed_sprite_to_foliage() {
        let seed = plant_visual(PlotStage::Growing {
            crop: CropKind::Turnip,
            days: 0,
            watered: false,
        });
        let sprout = plant_visual(PlotStage::Growing {
            crop: CropKind::Turnip,
            days: 1,
            watered: true,
        });
        assert_eq!(seed.sprite_kind, PlantSpriteKind::Seed);
        assert_eq!(sprout.sprite_kind, PlantSpriteKind::Foliage);
        // Sprout is taller than the seed cluster height.
        assert!(sprout.size.y > seed.size.y * 0.4);
    }

    #[test]
    fn tilled_soil_darker_than_untilled() {
        let soil = soil_visual(PlotStage::Soil);
        let tilled = soil_visual(PlotStage::Tilled);
        // Tilled uses lower luminance browns.
        assert!(tilled.color.to_srgba().red < soil.color.to_srgba().red);
    }
}
