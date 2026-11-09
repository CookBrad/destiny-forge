use bevy::prelude::*;

use crate::graphics::{world_transform, TILE};

#[derive(Clone)]
pub struct ZoneRect<Z> {
    pub zone: Z,
    pub bounds: Rect,
}

pub fn zone_at<Z>(zones: &[ZoneRect<Z>], position: Vec2) -> Option<&ZoneRect<Z>> {
    zones
        .iter()
        .rev()
        .find(|zone| zone.bounds.contains(position))
}

pub fn tile_rect(x0: u32, y0: u32, x1: u32, y1: u32) -> Rect {
    Rect {
        min: Vec2::new(x0 as f32 * TILE, y0 as f32 * TILE),
        max: Vec2::new(x1 as f32 * TILE, y1 as f32 * TILE),
    }
}

pub fn build_map_border(solids: &mut Vec<Rect>, map_tiles_w: u32, map_tiles_h: u32) {
    solids.push(tile_rect(0, 0, map_tiles_w, 1));
    solids.push(tile_rect(0, map_tiles_h - 1, map_tiles_w, map_tiles_h));
    solids.push(tile_rect(0, 0, 1, map_tiles_h));
    solids.push(tile_rect(map_tiles_w - 1, 0, map_tiles_w, map_tiles_h));
}

pub fn tile_checker_shade(tx: u32, ty: u32) -> f32 {
    if (tx + ty).is_multiple_of(2) {
        1.0
    } else {
        0.9
    }
}

pub fn tint_shade(color: Color, shade: f32) -> Color {
    let c = color.to_srgba();
    Color::srgba(c.red * shade, c.green * shade, c.blue * shade, c.alpha)
}

pub struct GridOverlayStyle {
    pub line_color: Color,
    pub z: f32,
}

pub struct GridOverlayExtent {
    pub world_width: f32,
    pub world_height: f32,
    pub map_tiles_w: u32,
    pub map_tiles_h: u32,
}

pub fn spawn_grid_overlay(
    commands: &mut Commands,
    grid_line: Handle<Image>,
    extent: GridOverlayExtent,
    style: GridOverlayStyle,
    mut decorate: impl FnMut(&mut EntityCommands),
) {
    let GridOverlayExtent {
        world_width,
        world_height,
        map_tiles_w,
        map_tiles_h,
    } = extent;
    for tx in 0..=map_tiles_w {
        let x = tx as f32 * TILE;
        let mut entity = commands.spawn((
            Sprite {
                image: grid_line.clone(),
                color: style.line_color,
                // Stroke length is a world-unit fill. The image is a 1×1 pixel, not tile art.
                custom_size: Some(Vec2::new(1.0, world_height)),
                ..default()
            },
            world_transform(Vec2::new(x, world_height * 0.5), style.z),
        ));
        decorate(&mut entity);
    }

    for ty in 0..=map_tiles_h {
        let y = ty as f32 * TILE;
        let mut entity = commands.spawn((
            Sprite {
                image: grid_line.clone(),
                color: style.line_color,
                // Stroke length is a world-unit fill. The image is a 1×1 pixel, not tile art.
                custom_size: Some(Vec2::new(world_width, 1.0)),
                ..default()
            },
            world_transform(Vec2::new(world_width * 0.5, y), style.z),
        ));
        decorate(&mut entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRID_STROKE: f32 = 1.0;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SampleZone {
        Field,
        Gate,
    }

    #[test]
    fn a_position_inside_a_zone_returns_that_zone() {
        let zones = [ZoneRect {
            zone: SampleZone::Field,
            bounds: tile_rect(0, 0, 2, 2),
        }];
        let found = zone_at(&zones, Vec2::new(TILE * 0.5, TILE * 0.5)).unwrap();
        assert_eq!(found.zone, SampleZone::Field);
    }

    #[test]
    fn the_later_zone_wins_when_two_zones_overlap() {
        let zones = [
            ZoneRect {
                zone: SampleZone::Field,
                bounds: tile_rect(0, 0, 4, 4),
            },
            ZoneRect {
                zone: SampleZone::Gate,
                bounds: tile_rect(1, 1, 3, 3),
            },
        ];
        assert_eq!(
            zone_at(&zones, Vec2::new(TILE * 0.5, TILE * 0.5))
                .unwrap()
                .zone,
            SampleZone::Field
        );
        assert_eq!(
            zone_at(&zones, Vec2::new(TILE * 2.0, TILE * 2.0))
                .unwrap()
                .zone,
            SampleZone::Gate
        );
    }

    #[test]
    fn a_position_outside_every_zone_returns_none() {
        let zones = [ZoneRect {
            zone: SampleZone::Field,
            bounds: tile_rect(0, 0, 2, 2),
        }];
        assert!(zone_at(&zones, Vec2::new(TILE * 3.0, TILE * 0.5)).is_none());
        assert!(zone_at::<SampleZone>(&[], Vec2::new(TILE * 0.5, TILE * 0.5)).is_none());
    }

    #[derive(Component)]
    struct OverlayLine;

    #[test]
    fn a_two_by_one_overlay_draws_every_grid_line() {
        let world_width = TILE * 2.0;
        let world_height = TILE;
        let map_tiles_w = 2;
        let map_tiles_h = 1;
        let color = Color::srgb(0.2, 0.4, 0.6);
        let z = 3.0;
        let lines = overlay_lines(
            world_width,
            world_height,
            map_tiles_w,
            map_tiles_h,
            color,
            z,
        );
        assert_eq!(
            lines.len(),
            map_tiles_w as usize + 1 + map_tiles_h as usize + 1
        );
        let vertical = lines
            .iter()
            .find(|(size, _, _)| *size == Some(Vec2::new(GRID_STROKE, world_height)))
            .unwrap();
        assert_eq!(
            vertical.1,
            world_transform(Vec2::new(0.0, world_height * 0.5), z)
        );
        assert_eq!(vertical.2, color);
        let horizontal = lines
            .iter()
            .find(|(size, _, _)| *size == Some(Vec2::new(world_width, GRID_STROKE)))
            .unwrap();
        assert_eq!(
            horizontal.1,
            world_transform(Vec2::new(world_width * 0.5, 0.0), z)
        );
        assert_eq!(horizontal.2, color);
    }

    fn overlay_lines(
        world_width: f32,
        world_height: f32,
        map_tiles_w: u32,
        map_tiles_h: u32,
        color: Color,
        z: f32,
    ) -> Vec<(Option<Vec2>, Transform, Color)> {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::default());
        app.add_systems(Update, move |mut commands: Commands| {
            spawn_grid_overlay(
                &mut commands,
                image.clone(),
                GridOverlayExtent {
                    world_width,
                    world_height,
                    map_tiles_w,
                    map_tiles_h,
                },
                GridOverlayStyle {
                    line_color: color,
                    z,
                },
                |entity| {
                    entity.insert(OverlayLine);
                },
            );
        });
        app.update();
        let mut lines = Vec::new();
        let mut drawn = app
            .world_mut()
            .query::<(&Sprite, &Transform, &OverlayLine)>();
        for (sprite, transform, _) in drawn.iter(app.world()) {
            lines.push((sprite.custom_size, *transform, sprite.color));
        }
        lines
    }

    #[test]
    fn an_empty_map_still_draws_the_origin_cross() {
        let lines = overlay_lines(0.0, 0.0, 0, 0, Color::WHITE, 1.0);
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().any(|(size, transform, color)| {
            *size == Some(Vec2::new(GRID_STROKE, 0.0))
                && *transform == world_transform(Vec2::new(0.0, 0.0), 1.0)
                && *color == Color::WHITE
        }));
        assert!(lines.iter().any(|(size, transform, color)| {
            *size == Some(Vec2::new(0.0, GRID_STROKE))
                && *transform == world_transform(Vec2::ZERO, 1.0)
                && *color == Color::WHITE
        }));
    }

    #[test]
    fn a_two_tile_overlay_does_not_draw_a_third_column() {
        let world_width = TILE * 2.0;
        let world_height = TILE;
        let z = 3.0;
        let lines = overlay_lines(world_width, world_height, 2, 1, Color::WHITE, z);
        let past_the_map = world_transform(Vec2::new(TILE * 3.0, world_height * 0.5), z);
        assert!(lines
            .iter()
            .all(|(_, transform, _)| *transform != past_the_map));
    }
}
