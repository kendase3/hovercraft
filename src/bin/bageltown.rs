// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bevy::render::camera::ScalingMode;
/// Currently more or less a mashup of existing tutorials, but one day!
use bevy::{core_pipeline::bloom::Bloom, prelude::*, text::FontSmoothing};

#[derive(Component)]
struct Player;

const MOVE_PER_TICK: f32 = 40.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // fill whole browser window
                fit_canvas_to_parent: true,
                // don't listen to keyboard shortcuts like F keys, ctrl+R
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.53, 0.53, 0.53)))
        .add_systems(Startup, startup)
        .add_systems(Update, move_player)
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true, // HDR is required for the bloom effect
            ..default()
        },
        Bloom::NATURAL,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 200.,
            },
            // This is the default value for scale for orthographic projections.
            // To zoom in and out, change this value, rather than `ScalingMode` or the camera's position.
            scale: 1.,
            ..OrthographicProjection::default_2d()
        }),
    ));
    let player = meshes.add(Circle::new(20.));
    let color = Color::srgb(0.0, 0.0, 0.0);
    commands.spawn((
        Player,
        Mesh2d(player),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    // FIXME(skend): dink around with text stuff here
    let font = asset_server.load("fonts/DejaVuSansMono.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 30.0,
        ..default()
    };
    commands.spawn((
        Text2d::new("@"),
        text_font
            .clone()
            .with_font_smoothing(FontSmoothing::AntiAliased),
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::srgb(1., 0.0, 1.)),
        Player,
    ));
}
fn move_player(
    mut players: Query<&mut Transform, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut direction = Vec2::ZERO;
    if keys.any_pressed([KeyCode::KeyW]) {
        direction.y += 1.;
    }
    if keys.any_pressed([KeyCode::KeyS]) {
        direction.y -= 1.;
    }
    if keys.any_pressed([KeyCode::KeyD]) {
        direction.x += 1.;
    }
    if keys.any_pressed([KeyCode::KeyA]) {
        direction.x -= 1.;
    }

    let move_speed = MOVE_PER_TICK;
    let move_delta = direction * move_speed * time.delta_secs();

    for mut transform in &mut players {
        transform.translation += move_delta.extend(0.);
    }
}
