//! Crop plot entities and visuals on the homestead.

use bevy::prelude::*;

use crate::graphics::{world_transform, TILE};
use crate::overworld::layout::{tile_center, OverworldEntity};
use crate::overworld::sprites::OverworldArt;

use super::crops::{advance_plot_day, PlotStage};

#[derive(Component, Clone, Debug)]
pub struct CropPlot {
    pub tile_x: u32,
    pub tile_y: u32,
    pub stage: PlotStage,
}

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
            // Sparse grid of workable plots.
            if (tx + ty) % 2 != 0 {
                continue;
            }
            let center = tile_center(tx, ty);
            let stage = PlotStage::Soil;
            commands.spawn((
                Sprite {
                    image: art.soil.clone(),
                    color: plot_color(stage),
                    custom_size: Some(Vec2::splat(TILE * 0.9)),
                    ..default()
                },
                world_transform(center, 1.15),
                CropPlot {
                    tile_x: tx,
                    tile_y: ty,
                    stage,
                },
                OverworldEntity,
            ));
        }
    }
}

pub fn plot_color(stage: PlotStage) -> Color {
    match stage {
        PlotStage::Soil => Color::srgb(0.38, 0.26, 0.16),
        PlotStage::Tilled => Color::srgb(0.28, 0.18, 0.1),
        PlotStage::Growing {
            watered: true,
            days,
            ..
        } => {
            let t = (days as f32 * 0.15).min(0.5);
            Color::srgb(0.22 + t, 0.55 + t * 0.2, 0.28)
        }
        PlotStage::Growing {
            watered: false, ..
        } => Color::srgb(0.32, 0.48, 0.24),
        PlotStage::Ready { .. } => Color::srgb(0.82, 0.72, 0.28),
    }
}

pub fn sync_plot_visuals(mut plots: Query<(&CropPlot, &mut Sprite), Changed<CropPlot>>) {
    for (plot, mut sprite) in &mut plots {
        sprite.color = plot_color(plot.stage);
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
