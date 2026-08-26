use bevy::prelude::*;

use crate::graphics::{center_on_surface, world_transform, TILE};

use super::layout::{
    homestead_forest_transition, homestead_lake_transition, spawn_homestead, tile_center,
    OverworldLayout, WORLD_WIDTH,
};
use super::movement::{ExplorationMap, MapTransitionCooldown, OverworldPlayer};
use super::sprites::{OverworldArt, PLAYER_SPRITE_HEIGHT};

/// Where the player appears when entering the homestead.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub enum OverworldEntry {
    Yard,
    /// Exact world position (feet/center XY) — used when returning from forest/lake exits.
    At(Vec2),
    DungeonReturn,
}

impl Default for OverworldEntry {
    fn default() -> Self {
        Self::Yard
    }
}

impl OverworldEntry {
    /// Spawn just inside the forest trail, aligned with the exit band.
    pub fn from_forest_return(player_pos_in_forest_exit: Vec2) -> Self {
        let home = homestead_forest_transition();
        // Forest exit is the south edge of the forest trail; map X across the band.
        let forest_exit = crate::forest::layout::forest_homestead_transition();
        let t = if forest_exit.width() > 1.0 {
            ((player_pos_in_forest_exit.x - forest_exit.min.x) / forest_exit.width()).clamp(0.0, 1.0)
        } else {
            0.5
        };
        let x = home.min.x + t * home.width();
        // One tile south of the north transition so we don't immediately re-enter.
        let y = home.min.y - TILE * 0.75;
        Self::At(Vec2::new(x, y.max(TILE * 2.0)))
    }

    /// Spawn just inside the east lake trail, aligned with the exit band.
    pub fn from_lake_return(player_pos_in_lake_exit: Vec2) -> Self {
        let home = homestead_lake_transition();
        let lake_exit = crate::lake::layout::lake_homestead_transition();
        let t = if lake_exit.height() > 1.0 {
            ((player_pos_in_lake_exit.y - lake_exit.min.y) / lake_exit.height()).clamp(0.0, 1.0)
        } else {
            0.5
        };
        let y = home.min.y + t * home.height();
        // One tile west of the east transition so we don't immediately re-enter.
        let x = home.min.x - TILE * 0.75;
        Self::At(Vec2::new(x.max(TILE * 2.0), y))
    }

    pub fn spawn_pos(self) -> Vec2 {
        match self {
            Self::Yard => Vec2::new(WORLD_WIDTH * 0.5, TILE * 12.0),
            Self::At(pos) => pos,
            Self::DungeonReturn => tile_center(25, 4),
        }
    }
}

pub fn setup_overworld(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    entry: Option<Res<OverworldEntry>>,
) {
    let art = OverworldArt::load(&asset_server, &mut atlas_layouts);
    let layout = OverworldLayout::homestead();
    let spawn = entry.map(|entry| *entry).unwrap_or_default();

    spawn_homestead(&mut commands, &art);
    spawn_overworld_player(&mut commands, &art, spawn);

    commands.insert_resource(ExplorationMap {
        solids: layout.solids.clone(),
        world_width: super::layout::WORLD_WIDTH,
        world_height: super::layout::WORLD_HEIGHT,
    });
    commands.insert_resource(MapTransitionCooldown::default());
    commands.insert_resource(art);
    commands.insert_resource(layout);
    commands.remove_resource::<OverworldEntry>();
}

fn spawn_overworld_player(commands: &mut Commands, art: &OverworldArt, entry: OverworldEntry) {
    let start = entry.spawn_pos();
    // `At` stores sprite-center Y from the last session; other entries use surface tiles.
    let y = match entry {
        OverworldEntry::At(_) => start.y,
        _ => center_on_surface(start.y, PLAYER_SPRITE_HEIGHT),
    };

    commands.spawn((
        Sprite {
            image: art.player.idle[0].clone(),
            ..default()
        },
        world_transform(Vec2::new(start.x, y), 5.0),
        OverworldPlayer,
        super::movement::OverworldVelocity::default(),
        crate::farming::PlayerFacing::default(),
    ));
}

pub fn cleanup_overworld(
    mut commands: Commands,
    entities: Query<Entity, With<super::layout::OverworldEntity>>,
    players: Query<Entity, With<OverworldPlayer>>,
) {
    for entity in entities.iter().chain(players.iter()) {
        commands.entity(entity).try_despawn_recursive();
    }
    commands.remove_resource::<OverworldArt>();
    commands.remove_resource::<OverworldLayout>();
    commands.remove_resource::<ExplorationMap>();
    commands.remove_resource::<MapTransitionCooldown>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overworld::layout::{homestead_forest_transition, homestead_lake_transition};

    #[test]
    fn lake_return_spawns_inside_homestead_not_in_transition() {
        let lake_exit = crate::lake::layout::lake_homestead_transition();
        let mid = lake_exit.center();
        let entry = OverworldEntry::from_lake_return(mid);
        let pos = entry.spawn_pos();
        let home_exit = homestead_lake_transition();
        // Should be west of the transition strip (not still in the edge)
        assert!(pos.x < home_exit.min.x);
        // Y stays within the trail band
        assert!(pos.y >= home_exit.min.y - TILE);
        assert!(pos.y <= home_exit.max.y + TILE);
    }

    #[test]
    fn forest_return_spawns_inside_homestead_not_in_transition() {
        let forest_exit = crate::forest::layout::forest_homestead_transition();
        let mid = forest_exit.center();
        let entry = OverworldEntry::from_forest_return(mid);
        let pos = entry.spawn_pos();
        let home_exit = homestead_forest_transition();
        assert!(pos.y < home_exit.min.y);
        assert!(pos.x >= home_exit.min.x - TILE);
        assert!(pos.x <= home_exit.max.x + TILE);
    }
}
