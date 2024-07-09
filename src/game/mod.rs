//! Game mechanics and content.
//!
//! The basic movement code shipped with the template is based on the
//! corresponding [Bevy example](https://github.com/janhohenheim/bevy/blob/fixed-time-movement/examples/movement/physics_in_fixed_timestep.rs).
//! See that link for an in-depth explanation of the code and the motivation
//! behind it.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod movement;
mod physics;
mod render;
pub mod spawn;

pub(super) fn plugin(app: &mut App) {
    app.configure_sets(
        Update,
        (GameSystem::UpdateTransform, GameSystem::ReadInput).chain(),
    );
    app.add_plugins((
        movement::plugin,
        physics::plugin,
        render::plugin,
        spawn::plugin,
    ));
    app.register_type::<GameSystem>();
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, Reflect, Serialize, Deserialize)]
#[reflect(Debug, Hash, PartialEq, Serialize, Deserialize)]
enum GameSystem {
    /// Updates the [`Transform`] of entities based on their
    /// [`physics::PhysicalTransform`].
    UpdateTransform,
    /// Reads input from the player.
    ReadInput,
}
