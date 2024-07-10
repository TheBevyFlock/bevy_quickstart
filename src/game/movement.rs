//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/pull/14223).

use bevy::prelude::*;

use super::audio::sfx::Sfx;
use crate::AppStep;

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.register_type::<MovementController>();
    app.add_systems(
        Update,
        record_movement_controller.in_set(AppStep::RecordInput),
    );

    // Apply movement based on controls.
    app.register_type::<Movement>();
    app.add_systems(Update, apply_movement.in_set(AppStep::Update));

    // Update facing based on controls.
    app.add_systems(Update, update_facing.in_set(AppStep::Update));

    // Trigger step sound effects based on controls.
    app.register_type::<StepSfx>();
    app.add_systems(
        Update,
        (
            tick_step_sfx.in_set(AppStep::TickTimers),
            trigger_step_sfx.in_set(AppStep::Update),
        ),
    );
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementController(pub Vec2);

fn record_movement_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController>,
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

    // Normalize so that diagonal movement has the same speed as
    // horizontal and vertical movement.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.0 = intent;
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Movement {
    /// Since Bevy's default 2D camera setup is scaled such that
    /// one unit is one pixel, you can think of this as
    /// "How many pixels per second should the player move?"
    /// Note that physics engines may use different unit/pixel ratios.
    pub speed: f32,
}

fn apply_movement(
    time: Res<Time>,
    mut mobility_query: Query<(&MovementController, &Movement, &mut Transform)>,
) {
    for (controller, mobility, mut transform) in &mut mobility_query {
        let velocity = mobility.speed * controller.0;
        let velocity = velocity.extend(0.0);

        transform.translation += velocity * time.delta_seconds();
    }
}

fn update_facing(mut player_query: Query<(&MovementController, &mut Sprite)>) {
    for (controller, mut sprite) in &mut player_query {
        let dx = controller.0.x;
        if dx != 0.0 {
            sprite.flip_x = dx < 0.0;
        }
    }
}

/// Time between walk sound effects.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct StepSfx(pub Timer);

fn tick_step_sfx(time: Res<Time>, mut step_query: Query<&mut StepSfx>) {
    for mut step in &mut step_query {
        step.0.tick(time.delta());
    }
}

/// If the player is moving, play a step sound effect.
fn trigger_step_sfx(mut commands: Commands, step_query: Query<(&MovementController, &StepSfx)>) {
    for (controller, step) in &step_query {
        if controller.0 != Vec2::ZERO && step.0.just_finished() {
            commands.trigger(Sfx::Step);
        }
    }
}
