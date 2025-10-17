//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

use crate::game::tiled_map::TiledMap;

mod animation;
pub mod level;
pub mod map;
mod movement;
pub mod player;
pub mod tiled_map;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<TiledMap>();
    app.add_plugins((
        animation::plugin,
        level::plugin,
        movement::plugin,
        player::plugin,
        map::plugin,
        tiled_map::plugin,
    ));
}
