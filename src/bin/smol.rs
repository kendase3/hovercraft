// Copyright 2025 Google LLC
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

use bevy::core_pipeline::bloom::Bloom;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::PresentMode;

const CAMERA_DEFAULT_SIZE: f32 = 100.;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // fill whole browser window
                        fit_canvas_to_parent: true,
                        // don't listen to keyboard shortcuts like F keys, ctrl+R
                        prevent_default_event_handling: false,
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.53, 0.53, 0.53)))
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true, // HDR is required for the bloom effect
            order: 1,
            ..default()
        },
        Bloom::NATURAL,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: CAMERA_DEFAULT_SIZE,
            },
            scale: 1.,
            ..OrthographicProjection::default_2d()
        }),
    ));
    let player_mesh = meshes.add(Circle::new(10.));
    let player_color = Color::srgb(0.0, 1.0, 0.7);
    commands.spawn((
        Player,
        Name::new("kewlplayer"),
        Mesh2d(player_mesh),
        MeshMaterial2d(materials.add(player_color)),
        // you could put it somewhere special if you wanted
        Transform::default(),
    ));
}

fn move_player(
    mut players: Query<&mut Transform, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // we happen to know the query will just return one thing
    let mut player_transform = players.single_mut();
    let mut direction = Vec3::ZERO;
    if keys.any_pressed([KeyCode::KeyW]) {
        direction.y += 1.0;
    }
    if keys.any_pressed([KeyCode::KeyS]) {
        direction.y -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyD]) {
        direction.x += 1.0;
    }
    if keys.any_pressed([KeyCode::KeyA]) {
        direction.x -= 1.0;
    }
    let move_speed = 2.0;
    player_transform.translation += direction * move_speed;
}
