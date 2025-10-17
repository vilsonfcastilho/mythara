//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{AppSystems, PausableSystems, game::tiled_map::CollisionTiles};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (apply_movement, apply_screen_wrap)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub max_speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            // 400 pixels per second is a nice default, but we can still vary this per character.
            max_speed: 400.0,
        }
    }
}

fn apply_movement(
    time: Res<Time>,
    collisions: Res<CollisionTiles>,
    mut movement_query: Query<(&MovementController, &mut Transform)>,
) {
    for (controller, mut transform) in &mut movement_query {
        let velocity = controller.max_speed * controller.intent;

        if velocity.length_squared() == 0.0 || collisions.blocked.is_empty() {
            transform.translation += velocity.extend(0.0) * time.delta_secs();
            continue;
        }

        let current = transform.translation.xy();
        let target = current + velocity * time.delta_secs();

        // If target tile is blocked, prevent movement this frame
        let target_tile = world_to_iso_tile(target, &collisions);
        if collisions.blocked.contains(&target_tile) {
            continue;
        }

        transform.translation = target.extend(transform.translation.z);
    }
}

fn world_to_iso_tile(world: Vec2, collisions: &CollisionTiles) -> IVec2 {
    let half_w = collisions.grid_size.x * 0.5;
    let half_h = collisions.grid_size.y * 0.5;

    // Center of the map in tile coordinates (due to TilemapAnchor::Center)
    let center_x = (collisions.map_size.x as f32 - 1.0) * 0.5;
    let center_y = (collisions.map_size.y as f32 - 1.0) * 0.5;

    // Undo tilemap transform (offset applied at spawn)
    let local = world - collisions.layer_offset;

    // Convert to skewed isometric space
    let sx = local.x / half_w;
    let sy = local.y / half_h;

    // Inverse of:
    // world.x = (x - y - (center_x - center_y)) * half_w + offset_x
    // world.y = (x + y - (center_x + center_y)) * half_h - offset_y
    let tx = (sy + sx) * 0.5 + center_x;
    let ty = (sy - sx) * 0.5 + center_y;

    IVec2::new(tx.floor() as i32, ty.floor() as i32)
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScreenWrap;

fn apply_screen_wrap(
    window: Single<&Window, With<PrimaryWindow>>,
    mut wrap_query: Query<&mut Transform, With<ScreenWrap>>,
) {
    let size = window.size() + 256.0;
    let half_size = size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
}
