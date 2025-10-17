use std::{
    collections::{HashMap, HashSet},
    io::{Cursor, ErrorKind},
    sync::Arc,
};

use bevy::{
    asset::{AssetLoader, AssetPath},
    prelude::*,
};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_ecs_tilemap::prelude::*;
use thiserror::Error;

pub(super) fn plugin(app: &mut App) {
    app.register_asset_loader(TiledLoader);
    app.add_plugins(TilemapPlugin);
    app.init_resource::<CollisionTiles>();
    app.add_systems(Update, process_loaded_maps);
}

#[derive(TypePath, Asset)]
pub struct TiledMap {
    pub map: tiled::Map,
    pub tilemap_textures: HashMap<usize, TilemapTexture>,
}

#[derive(Default, Component, Debug)]
pub struct TiledLayersStorage {
    pub storage: HashMap<u32, Entity>,
}

#[derive(Default, Component)]
pub struct TiledMapHandle(pub Handle<TiledMap>);

#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: TiledMapHandle,
    pub storage: TiledLayersStorage,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub render_settings: TilemapRenderSettings,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct CollisionTiles {
    pub blocked: HashSet<IVec2>,
    pub map_size: UVec2,
    pub grid_size: Vec2,
    pub layer_offset: Vec2,
}

pub struct BytesResourceReader {
    bytes: Arc<[u8]>,
}

impl BytesResourceReader {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: Arc::from(bytes),
        }
    }
}

impl tiled::ResourceReader for BytesResourceReader {
    type Resource = Cursor<Arc<[u8]>>;
    type Error = std::io::Error;

    fn read_from(
        &mut self,
        _path: &std::path::Path, // In this case, the path is ignored because the byte data is already provided.
    ) -> std::result::Result<Self::Resource, Self::Error> {
        Ok(Cursor::new(self.bytes.clone()))
    }
}

#[derive(Debug, Error)]
pub enum TiledAssetLoaderError {
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
}

pub struct TiledLoader;

impl AssetLoader for TiledLoader {
    type Asset = TiledMap;
    type Settings = ();
    type Error = TiledAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> std::result::Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let mut loader = tiled::Loader::with_cache_and_reader(
            tiled::DefaultResourceCache::new(),
            BytesResourceReader::new(&bytes),
        );

        let map = loader.load_tmx_map(load_context.path()).map_err(|e| {
            std::io::Error::new(ErrorKind::Other, format!("Could not load TMX map: {e}"))
        })?;

        let mut tilemap_textures = HashMap::default();

        for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
            let tilemap_texture = match &tileset.image {
                None => {
                    info!("Unsupported tileset type {}", tileset.name);
                    continue;
                }
                Some(img) => {
                    // The load context path is the TMX file itself. If the file is at the root of the
                    // assets/ directory structure then the tmx_dir will be empty, which is fine.
                    // let tmx_dir = load_context
                    //     .path()
                    //     .parent()
                    //     .expect("The asset load context was empty.");
                    // let tile_path = tmx_dir.join(&img.source);
                    // let asset_path = AssetPath::from(tile_path);
                    let asset_path = AssetPath::from(img.source.clone());
                    let texture: Handle<Image> = load_context.load(asset_path.clone());

                    TilemapTexture::Single(texture.clone())
                }
            };

            tilemap_textures.insert(tileset_index, tilemap_texture);
        }

        let asset_map = TiledMap {
            map,
            tilemap_textures,
        };

        info!("Loaded map: {}", load_context.path().display());
        Ok(asset_map)
    }
}

pub fn process_loaded_maps(
    mut commands: Commands,
    mut map_events: MessageReader<AssetEvent<TiledMap>>,
    maps: Res<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut map_query: Query<(
        &TiledMapHandle,
        &mut TiledLayersStorage,
        &mut TilemapRenderSettings,
    )>,
    new_maps: Query<&TiledMapHandle, Added<TiledMapHandle>>,
    mut collisions: ResMut<CollisionTiles>,
) {
    let mut changed_maps = Vec::<AssetId<TiledMap>>::default();
    for event in map_events.read() {
        match event {
            AssetEvent::Added { id } => {
                info!("Map added!");
                changed_maps.push(*id);
            }
            AssetEvent::Modified { id } => {
                info!("Map changed!");
                changed_maps.push(*id);
            }
            AssetEvent::Removed { id } => {
                info!("Map removed!");
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                changed_maps.retain(|changed_handle| changed_handle == id);
            }
            _ => continue,
        }
    }

    // If we have new map entities add them to the changed_maps list.
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.0.id());
    }

    for changed_map in changed_maps.iter() {
        for (map_handle, mut layer_storage, mut render_settings) in map_query.iter_mut() {
            // only deal with currently changed map
            if map_handle.0.id() != *changed_map {
                continue;
            }
            if let Some(tiled_map) = maps.get(&map_handle.0) {
                // TODO: Create a RemoveMap component..
                for layer_entity in layer_storage.storage.values() {
                    if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
                        for tile in layer_tile_storage.iter().flatten() {
                            commands.entity(*tile).despawn()
                        }
                    }
                    // commands.entity(*layer_entity).despawn_recursive();
                }

                // No overlay entities to clean up when tinting directly

                // The TilemapBundle requires that all tile images come exclusively from a single
                // tiled texture or from a Vec of independent per-tile images. Furthermore, all of
                // the per-tile images must be the same size. Since Tiled allows tiles of mixed
                // tilesets on each layer and allows differently-sized tile images in each tileset,
                // this means we need to load each combination of tileset and layer separately.
                for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
                    let Some(tilemap_texture) = tiled_map.tilemap_textures.get(&tileset_index)
                    else {
                        warn!("Skipped creating layer with missing tilemap textures.");
                        continue;
                    };

                    let tile_size = TilemapTileSize {
                        x: tileset.tile_width as f32,
                        y: tileset.tile_height as f32,
                    };

                    let tile_spacing = TilemapSpacing {
                        x: tileset.spacing as f32,
                        y: tileset.spacing as f32,
                    };

                    // Once materials have been created/added we need to then create the layers.
                    for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                        let offset_x = layer.offset_x;
                        let offset_y = layer.offset_y;

                        let tiled::LayerType::Tiles(tile_layer) = layer.layer_type() else {
                            info!(
                                "Skipping layer {} because only tile layers are supported.",
                                layer.id()
                            );
                            continue;
                        };

                        let tiled::TileLayer::Finite(layer_data) = tile_layer else {
                            info!(
                                "Skipping layer {} because only finite layers are supported.",
                                layer.id()
                            );
                            continue;
                        };

                        let map_size = TilemapSize {
                            x: tiled_map.map.width,
                            y: tiled_map.map.height,
                        };

                        let grid_size = TilemapGridSize {
                            x: tiled_map.map.tile_width as f32,
                            y: tiled_map.map.tile_height as f32,
                        };

                        let map_type = match tiled_map.map.orientation {
                            tiled::Orientation::Hexagonal => {
                                TilemapType::Hexagon(HexCoordSystem::Row)
                            }
                            tiled::Orientation::Isometric => {
                                TilemapType::Isometric(IsoCoordSystem::Diamond)
                            }
                            tiled::Orientation::Staggered => {
                                TilemapType::Isometric(IsoCoordSystem::Staggered)
                            }
                            tiled::Orientation::Orthogonal => TilemapType::Square,
                        };

                        let mut tile_storage = TileStorage::empty(map_size);
                        let layer_entity = commands.spawn_empty().id();

                        // If this is the Collisions layer, rebuild the collision set
                        let is_collision_layer = layer.name == "Collisions";
                        if is_collision_layer {
                            collisions.blocked.clear();
                            collisions.map_size = UVec2::new(map_size.x, map_size.y);
                            collisions.grid_size = Vec2::new(
                                tiled_map.map.tile_width as f32,
                                tiled_map.map.tile_height as f32,
                            );
                            collisions.layer_offset = Vec2::new(offset_x, -offset_y);
                        }

                        for x in 0..map_size.x {
                            for y in 0..map_size.y {
                                // Transform TMX coords into bevy coords.
                                let mapped_y = tiled_map.map.height - 1 - y;

                                let mapped_x = x as i32;
                                let mapped_y = mapped_y as i32;

                                let layer_tile = match layer_data.get_tile(mapped_x, mapped_y) {
                                    Some(t) => t,
                                    None => {
                                        continue;
                                    }
                                };
                                if tileset_index != layer_tile.tileset_index() {
                                    continue;
                                }

                                let layer_tile_data =
                                    match layer_data.get_tile_data(mapped_x, mapped_y) {
                                        Some(d) => d,
                                        None => {
                                            continue;
                                        }
                                    };

                                let texture_index = match tilemap_texture {
                                    TilemapTexture::Single(_) => layer_tile.id(),
                                    _ => unreachable!(),
                                };

                                let tile_pos = TilePos { x, y };
                                let tile_entity = commands
                                    .spawn(TileBundle {
                                        position: tile_pos,
                                        tilemap_id: TilemapId(layer_entity),
                                        texture_index: TileTextureIndex(texture_index),
                                        flip: TileFlip {
                                            x: layer_tile_data.flip_h,
                                            y: layer_tile_data.flip_v,
                                            d: layer_tile_data.flip_d,
                                        },
                                        ..Default::default()
                                    })
                                    .id();

                                tile_storage.set(&tile_pos, tile_entity);

                                // Record collision tiles by logical map coordinates
                                if is_collision_layer {
                                    // Rotate left (90° CCW) to align collision sampling with visuals
                                    let width_i = map_size.x as i32;
                                    let rotated = IVec2::new(y as i32, width_i - 1 - x as i32);
                                    collisions.blocked.insert(rotated);
                                }
                            }
                        }

                        if map_type == TilemapType::Isometric(IsoCoordSystem::Diamond) {
                            render_settings.render_chunk_size = UVec2::new(1, 1);
                            render_settings.y_sort = true;
                        }

                        commands.entity(layer_entity).insert(TilemapBundle {
                            grid_size,
                            size: map_size,
                            storage: tile_storage,
                            texture: tilemap_texture.clone(),
                            tile_size,
                            spacing: tile_spacing,
                            anchor: TilemapAnchor::Center,
                            transform: Transform::from_xyz(offset_x, -offset_y, layer_index as f32),
                            map_type,
                            render_settings: *render_settings,
                            ..Default::default()
                        });

                        layer_storage
                            .storage
                            .insert(layer_index as u32, layer_entity);

                        // No overlay tilemap needed when tinting directly
                    }
                }
            }
        }
    }
}
