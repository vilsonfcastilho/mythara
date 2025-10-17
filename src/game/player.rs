//! Player-specific behavior.

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        animation::PlayerAnimation,
        movement::{MovementController, ScreenWrap},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (record_player_directional_input)
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );

    // Make the main 2D camera follow the player.
    app.add_systems(
        Update,
        follow_player_camera
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// The player character.
pub fn player(
    max_speed: f32,
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::new(96, 80), 8, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    (
        Name::new("Player"),
        Player,
        Sprite::from_atlas_image(
            player_assets.spritesheets[2].clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.get_atlas_index(),
            },
        ),
        Transform {
            translation: Vec3::new(0., 16., 3.),
            scale: Vec2::splat(1.0).extend(1.0),
            ..Default::default()
        },
        MovementController {
            max_speed,
            ..default()
        },
        ScreenWrap,
        player_animation,
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Player;

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
    // This should be omitted if the input comes from an analog stick instead.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.intent = intent;
    }
}

fn follow_player_camera(
    player_transform: Single<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    mut mouse_wheel: MessageReader<MouseWheel>,
) {
    // Accumulate scroll input this frame
    let mut scroll = 0.0f32;
    for ev in mouse_wheel.read() {
        let step = match ev.unit {
            MouseScrollUnit::Line => 0.1,
            MouseScrollUnit::Pixel => 0.001,
        };
        scroll += ev.y as f32 * step;
    }

    for mut cam_transform in &mut camera_query {
        // Follow player position
        cam_transform.translation.x = player_transform.translation.x;
        cam_transform.translation.y = player_transform.translation.y;

        // Apply zoom via camera transform scaling (scroll up -> zoom in)
        if scroll != 0.0 {
            let current = cam_transform.scale.x.max(0.0001);
            let target = (current * (1.0 - scroll)).clamp(0.25, 3.0);
            cam_transform.scale.x = target;
            cam_transform.scale.y = target;
            // Keep Z scale unchanged
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    pub spritesheets: Vec<Handle<Image>>,
    #[dependency]
    pub sounds: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            spritesheets: vec![
                // IDLE
                assets.load_with_settings(
                    "images/player/idle/idle_east.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Use `nearest` image sampling to preserve pixel art style.
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
                assets.load_with_settings(
                    "images/player/idle/idle_north.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Use `nearest` image sampling to preserve pixel art style.
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
                assets.load_with_settings(
                    "images/player/idle/idle_south.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Use `nearest` image sampling to preserve pixel art style.
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
                assets.load_with_settings(
                    "images/player/idle/idle_west.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Use `nearest` image sampling to preserve pixel art style.
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
                // RUN
                assets.load_with_settings(
                    "images/player/run/run_east.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Use `nearest` image sampling to preserve pixel art style.
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
                assets.load_with_settings(
                    "images/player/run/run_north.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Use `nearest` image sampling to preserve pixel art style.
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
                assets.load_with_settings(
                    "images/player/run/run_south.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Use `nearest` image sampling to preserve pixel art style.
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
                assets.load_with_settings(
                    "images/player/run/run_west.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Use `nearest` image sampling to preserve pixel art style.
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
                // ATTACK
                assets.load_with_settings(
                    "images/player/attack/attack_east.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Use `nearest` image sampling to preserve pixel art style.
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
                assets.load_with_settings(
                    "images/player/attack/attack_north.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Use `nearest` image sampling to preserve pixel art style.
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
                assets.load_with_settings(
                    "images/player/attack/attack_south.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Use `nearest` image sampling to preserve pixel art style.
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
                assets.load_with_settings(
                    "images/player/attack/attack_west.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Use `nearest` image sampling to preserve pixel art style.
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
            ],
            sounds: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
