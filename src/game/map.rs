//! Spawn the main map.

use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    game::tiled_map::{TiledMap, TiledMapBundle, TiledMapHandle},
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<MapAssets>();
}

/// A system that spawns the main map.
pub fn map(map_assets: &MapAssets) -> impl Bundle {
    let map_handle = map_assets.map.clone();

    (
        Name::new("Map"),
        TiledMapBundle {
            tiled_map: TiledMapHandle(map_handle),
            ..Default::default()
        },
    )
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct MapAssets {
    #[dependency]
    pub map: Handle<TiledMap>,
}

impl FromWorld for MapAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            map: assets.load("images/map/iso_map.tmx"),
        }
    }
}
