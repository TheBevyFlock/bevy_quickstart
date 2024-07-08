//! Handles spawning of entities. This is typically done through
//! [observers](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Observer.html).
// Alternatively this can be done through `Events<E>` or `Commands`.

use bevy::prelude::*;

pub(crate) mod level;
mod player;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((level::plugin, player::plugin));
}
