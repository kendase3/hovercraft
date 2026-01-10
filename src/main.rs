use bevy::camera::ScalingMode;
//use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
//use rand::Rng;
//use std::collections::HashMap;
//use std::f32::consts::PI;
//use std::time::Duration;

const CAMERA_DEFAULT_SIZE: f32 = 100.;
// height of the largest letter
const FONT_SIZE: f32 = 10.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Time::<Fixed>::from_hz(60.))
        .add_systems(FixedUpdate, iterate_world)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // TODO(skend): first how about displaying any ttf
    let font = asset_server.load("fonts/DejaVuSansMono.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: FONT_SIZE * 10.,
        font_smoothing: FontSmoothing::AntiAliased,
        ..default()
    };
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: CAMERA_DEFAULT_SIZE,
            },
            scale: 1.,
            ..OrthographicProjection::default_2d()
        }),
    ));
    commands.spawn((
        Text2d::new("@"),
        text_font.clone(),
        TextColor(Color::srgb(1., 0., 1.)),
        // NB(skend): needed to make font look prettier
        Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.1)),
    ));
}

fn iterate_world() {}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
