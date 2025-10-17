//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::prelude::*;
use rand::prelude::*;
use std::time::Duration;

use crate::{
    AppSystems, PausableSystems,
    audio::sound_effect,
    game::{movement::MovementController, player::PlayerAssets},
};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            (
                update_animation_movement,
                update_animation_atlas,
                trigger_step_sound_effect,
            )
                .chain()
                .run_if(resource_exists::<PlayerAssets>)
                .in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

/// Update the sprite direction and animation state (idling/walking).
fn update_animation_movement(mut player_query: Query<(&MovementController, &mut PlayerAnimation)>) {
    for (controller, mut animation) in &mut player_query {
        let dx = controller.intent.x;
        let dy = controller.intent.y;

        let mut animation_state = PlayerAnimationState::Idling;
        let mut animation_direction = animation.direction; // Default direction

        // Check if player is moving
        if controller.intent != Vec2::ZERO {
            animation_state = PlayerAnimationState::Running;

            // Determine direction based on movement
            // For diagonal movement, prioritize the axis with greater magnitude
            if dx.abs() > dy.abs() {
                // Horizontal movement takes priority
                if dx > 0.0 {
                    animation_direction = PlayerDirection::East;
                } else {
                    animation_direction = PlayerDirection::West;
                }
            } else {
                // Vertical movement takes priority
                if dy > 0.0 {
                    animation_direction = PlayerDirection::North;
                } else {
                    animation_direction = PlayerDirection::South;
                }
            }
        }

        animation.update_state_and_direction(animation_state, animation_direction);
    }
}

/// Update the animation timer.
fn update_animation_timer(time: Res<Time>, mut query: Query<&mut PlayerAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas(
    player_assets: Res<PlayerAssets>,
    mut query: Query<(&PlayerAnimation, &mut Sprite)>,
) {
    for (animation, mut sprite) in &mut query {
        // Update the frame index within the spritesheet
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = animation.get_atlas_index();
        }

        if animation.changed() {
            // Calculate the spritesheet index based on state and direction
            let spritesheet_index = match (animation.state, animation.direction) {
                (PlayerAnimationState::Idling, PlayerDirection::East) => 0,
                (PlayerAnimationState::Idling, PlayerDirection::North) => 1,
                (PlayerAnimationState::Idling, PlayerDirection::South) => 2,
                (PlayerAnimationState::Idling, PlayerDirection::West) => 3,
                (PlayerAnimationState::Running, PlayerDirection::East) => 4,
                (PlayerAnimationState::Running, PlayerDirection::North) => 5,
                (PlayerAnimationState::Running, PlayerDirection::South) => 6,
                (PlayerAnimationState::Running, PlayerDirection::West) => 7,
            };

            // Update the texture to use the correct spritesheet
            sprite.image = player_assets.spritesheets[spritesheet_index].clone();
        }
    }
}

/// If the player is moving, play a step sound effect synchronized with the
/// animation.
fn trigger_step_sound_effect(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut step_query: Query<&PlayerAnimation>,
) {
    for animation in &mut step_query {
        if animation.state == PlayerAnimationState::Running
            && animation.changed()
            && (animation.frame == 2 || animation.frame == 5)
        {
            let rng = &mut rand::rng();
            let random_step = player_assets.sounds.choose(rng).unwrap().clone();
            commands.spawn(sound_effect(random_step));
        }
    }
}

#[derive(Reflect, PartialEq, Copy, Clone)]
pub enum PlayerAnimationState {
    Idling,
    Running,
}

#[derive(Reflect, PartialEq, Copy, Clone)]
pub enum PlayerDirection {
    North,
    South,
    East,
    West,
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAnimation {
    timer: Timer,
    frame: usize,
    state: PlayerAnimationState,
    direction: PlayerDirection,
}

impl PlayerAnimation {
    /// The number of idle frames.
    const IDLE_FRAMES: usize = 8;
    /// The duration of each idle frame.
    const IDLE_INTERVAL: Duration = Duration::from_millis(100);
    /// The number of walking frames.
    const RUN_FRAMES: usize = 8;
    /// The duration of each walking frame.
    const RUN_INTERVAL: Duration = Duration::from_millis(50);

    fn idling_north() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Idling,
            direction: PlayerDirection::North,
        }
    }
    fn idling_south() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Idling,
            direction: PlayerDirection::South,
        }
    }
    fn idling_east() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Idling,
            direction: PlayerDirection::East,
        }
    }
    fn idling_west() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Idling,
            direction: PlayerDirection::West,
        }
    }

    fn running_north() -> Self {
        Self {
            timer: Timer::new(Self::RUN_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Running,
            direction: PlayerDirection::North,
        }
    }
    fn running_south() -> Self {
        Self {
            timer: Timer::new(Self::RUN_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Running,
            direction: PlayerDirection::South,
        }
    }
    fn running_east() -> Self {
        Self {
            timer: Timer::new(Self::RUN_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Running,
            direction: PlayerDirection::East,
        }
    }
    fn running_west() -> Self {
        Self {
            timer: Timer::new(Self::RUN_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Running,
            direction: PlayerDirection::West,
        }
    }

    pub fn new() -> Self {
        Self::idling_south()
    }

    /// Update animation timers.
    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.is_finished() {
            return;
        }
        self.frame = (self.frame + 1)
            % match self.state {
                PlayerAnimationState::Idling => Self::IDLE_FRAMES,
                PlayerAnimationState::Running => Self::RUN_FRAMES,
            };
    }

    /// Update animation state and direction if it changes.
    pub fn update_state_and_direction(
        &mut self,
        state: PlayerAnimationState,
        direction: PlayerDirection,
    ) {
        if self.state != state || self.direction != direction {
            match (state, direction) {
                (PlayerAnimationState::Idling, PlayerDirection::North) => {
                    *self = Self::idling_north()
                }
                (PlayerAnimationState::Idling, PlayerDirection::South) => {
                    *self = Self::idling_south()
                }
                (PlayerAnimationState::Idling, PlayerDirection::East) => {
                    *self = Self::idling_east()
                }
                (PlayerAnimationState::Idling, PlayerDirection::West) => {
                    *self = Self::idling_west()
                }
                (PlayerAnimationState::Running, PlayerDirection::North) => {
                    *self = Self::running_north()
                }
                (PlayerAnimationState::Running, PlayerDirection::South) => {
                    *self = Self::running_south()
                }
                (PlayerAnimationState::Running, PlayerDirection::East) => {
                    *self = Self::running_east()
                }
                (PlayerAnimationState::Running, PlayerDirection::West) => {
                    *self = Self::running_west()
                }
            }
        }
    }

    /// Whether animation changed this tick.
    pub fn changed(&self) -> bool {
        self.timer.is_finished()
    }

    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        match self.state {
            PlayerAnimationState::Idling => self.frame,
            PlayerAnimationState::Running => self.frame,
        }
    }
}
