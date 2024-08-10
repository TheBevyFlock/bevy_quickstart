//! A splash screen that plays briefly at startup.

use std::collections::VecDeque;

use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
};

use super::Screen;
use crate::{theme::prelude::*, AppSet};

pub(super) fn plugin(app: &mut App) {
    // Spawn splash screen.
    app.insert_resource(ClearColor(SPLASH_BACKGROUND_COLOR));
    app.init_resource::<SplashScreenImageList>();

    app.add_systems(OnEnter(Screen::Splash), spawn_splash);

    // Animate splash screen.
    app.add_systems(
        Update,
        (
            tick_fade_in_out.in_set(AppSet::TickTimers),
            apply_fade_in_out.in_set(AppSet::Update),
        )
            .run_if(in_state(Screen::Splash)),
    );

    // Add splash timer.
    app.register_type::<SplashTimer>();
    app.add_systems(OnEnter(Screen::Splash), insert_splash_timer);
    app.add_systems(OnExit(Screen::Splash), remove_splash_timer);
    app.add_systems(
        Update,
        (
            tick_splash_timer.in_set(AppSet::TickTimers),
            check_splash_timer.in_set(AppSet::Update),
        )
            .run_if(in_state(Screen::Splash)),
    );

    // exit the splash screen early if the player hits escape
    app.add_systems(
        Update,
        exit_splash_screen
            .run_if(input_just_pressed(KeyCode::Escape).and_then(in_state(Screen::Splash))),
    );
}

const SPLASH_BACKGROUND_COLOR: Color = Color::srgb(0.157, 0.157, 0.157);
const SPLASH_DURATION_SECS: f32 = 1.8;
const SPLASH_FADE_DURATION_SECS: f32 = 0.6;

#[derive(Component)]
struct SplashScreenContainer;

#[derive(Resource)]
struct SplashScreenImageList(VecDeque<&'static str>);

impl Default for SplashScreenImageList {
    fn default() -> Self {
        // Use paths here rather than an ImageHandle as the loading screen hasn't
        // run yet. To show your own splash images, replace or add here
        Self(VecDeque::from_iter(["images/splash.png"]))
    }
}

fn exit_splash_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Loading);
}

fn splash_image_bundle(asset_server: &AssetServer, path: &'static str) -> impl Bundle {
    (
        Name::new("Splash image"),
        ImageBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                width: Val::Percent(70.0),
                ..default()
            },
            image: UiImage::new(asset_server.load_with_settings(
                // This should be an embedded asset for instant loading, but that is
                // currently [broken on Windows Wasm builds](https://github.com/bevyengine/bevy/issues/14246).
                path,
                |settings: &mut ImageLoaderSettings| {
                    // Make an exception for the splash image in case
                    // `ImagePlugin::default_nearest()` is used for pixel art.
                    settings.sampler = ImageSampler::linear();
                },
            )),
            background_color: BackgroundColor(Color::srgba(0., 0., 0., 0.)),
            ..default()
        },
        UiImageFadeInOut {
            total_duration: SPLASH_DURATION_SECS,
            fade_duration: SPLASH_FADE_DURATION_SECS,
            t: 0.0,
        },
    )
}

fn spawn_splash(
    mut commands: Commands,
    splash_images: Res<SplashScreenImageList>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if splash_images.0.is_empty() {
        // if there are no splash images, go directly to the loading screen
        next_screen.set(Screen::Loading);
    } else {
        // spawn the UI root but wait for the first timer tick to show an image
        commands.ui_root().insert((
            Name::new("Splash screen container"),
            BackgroundColor(SPLASH_BACKGROUND_COLOR),
            StateScoped(Screen::Splash),
            SplashScreenContainer,
        ));
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct UiImageFadeInOut {
    /// Total duration in seconds.
    total_duration: f32,
    /// Fade duration in seconds.
    fade_duration: f32,
    /// Current progress in seconds, between 0 and [`Self::total_duration`].
    t: f32,
}

impl UiImageFadeInOut {
    fn alpha(&self) -> f32 {
        // Normalize by duration.
        let t = (self.t / self.total_duration).clamp(0.0, 1.0);
        let fade = self.fade_duration / self.total_duration;

        // Regular trapezoid-shaped graph, flat at the top with alpha = 1.0.
        ((1.0 - (2.0 * t - 1.0).abs()) / fade).min(1.0)
    }
}

fn tick_fade_in_out(time: Res<Time>, mut animation_query: Query<&mut UiImageFadeInOut>) {
    for mut anim in &mut animation_query {
        anim.t += time.delta_seconds();
    }
}

fn apply_fade_in_out(mut animation_query: Query<(&UiImageFadeInOut, &mut UiImage)>) {
    for (anim, mut image) in &mut animation_query {
        image.color.set_alpha(anim.alpha())
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct SplashTimer(Timer);

impl Default for SplashTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SPLASH_DURATION_SECS, TimerMode::Once))
    }
}

fn insert_splash_timer(mut commands: Commands) {
    commands.init_resource::<SplashTimer>();
}

fn remove_splash_timer(mut commands: Commands) {
    commands.remove_resource::<SplashTimer>();
}

fn tick_splash_timer(time: Res<Time>, mut timer: ResMut<SplashTimer>) {
    timer.0.tick(time.delta());
}

fn check_splash_timer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut timer: ResMut<SplashTimer>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut splash_images: ResMut<SplashScreenImageList>,
    containers: Query<Entity, With<SplashScreenContainer>>,
) {
    if !timer.0.just_finished() {
        return;
    }

    // try to get the next splash image. If there isn't one, we move on to the next screen
    if let Some(next_splash_image) = splash_images.0.pop_front() {
        // despawn other splash images
        if let Ok(container) = containers.get_single() {
            commands
                .entity(container)
                .despawn_descendants()
                .with_children(|parent| {
                    parent.spawn(splash_image_bundle(&asset_server, next_splash_image));
                });
        }

        // reset the timer
        timer.0.reset();
    } else {
        // no more splash screens, exit
        next_screen.set(Screen::Loading);
    }
}
