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

/// Currently more or less a mashup of existing tutorials, but one day!

//use bevy::{prelude::*, render::camera::ScalingMode};
use bevy::prelude::*;


#[derive(Component)]
struct Player;

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
        .add_systems(Startup, (startup, spawn_player))
        .add_systems(Update, move_player)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 50000.,
            ..Default::default()
        },
        ..default()
    });
    let camera_bundle = Camera3dBundle {
        transform: Transform::from_xyz(35., 35., 35.)
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    };
    commands.spawn(camera_bundle);
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        SceneBundle {
            scene: asset_server.load("bagel.glb#Scene0"),
            ..default()
        },
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

    let move_speed = 7.;
    let move_delta = direction * move_speed * time.delta_seconds();

    for mut transform in &mut players {
        transform.translation += move_delta.extend(0.);
    }
}
