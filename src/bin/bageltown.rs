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

use bevy::log::LogPlugin;
use bevy::render::camera::ScalingMode;
use bevy::window::PresentMode;
/// Currently more or less a mashup of existing tutorials, but one day!
use bevy::{core_pipeline::bloom::Bloom, prelude::*, text::FontSmoothing};

#[derive(Component)]
struct Player {
    it: bool,
}

#[derive(Component)]
struct Bot;

#[derive(Component)]
struct Proclamation;

const MOVE_PER_TICK: f32 = 40.;

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
        .add_systems(Startup, startup)
        .add_systems(Update, (move_player, move_bot))
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
                viewport_height: 100.,
            },
            // This is the default value for scale for orthographic projections.
            // To zoom in and out, change this value, rather than `ScalingMode` or the camera's position.
            scale: 1.,
            ..OrthographicProjection::default_2d()
        }),
    ));
    let player = meshes.add(Circle::new(10.));
    let color = Color::srgb(0.0, 0.0, 0.0);
    let font = asset_server.load("fonts/DejaVuSansMono.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 100.0,
        ..default()
    };
    commands
        .spawn((
            Player { it: false },
            Name::new("Protagonist"),
            Mesh2d(player),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d::new("@"),
                text_font
                    .clone()
                    .with_font_smoothing(FontSmoothing::AntiAliased),
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::srgb(1., 0., 1.)),
                Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.2)),
            ));
        });
    let bot = meshes.add(Circle::new(10.));
    commands
        .spawn((
            Bot,
            Name::new("Antagonist"),
            Mesh2d(bot),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(50.0, 0.0, 0.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d::new("@"),
                text_font
                    .clone()
                    .with_font_smoothing(FontSmoothing::AntiAliased),
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::srgb(1., 0., 0.)),
                Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_scale(Vec3::splat(0.2)),
            ));
        });
    // hypothetical UI
    // UI
    commands.spawn((
        Text::new("You're it!"),
        Proclamation,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        Visibility::Hidden,
    ));
}

/*
fn update_proclamation(
    mut proclamation: Query(<&mut Transform, With<Proclamation>>)) {

}
*/

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

fn move_bot(
    mut bot: Query<(&mut Bot, &mut Transform)>,
    mut player: Query<(&mut Player, &mut Transform), Without<Bot>>,
    time: Res<Time>,
) {
    // FIXME(i currently have the background as a separate entity i guess
    let (mut b, b_t) = bot.single_mut();
    let (mut p, p_t) = player.single_mut();
    // find our position in x
    let x_delta = b_t.translation.x - p_t.translation.x;
    let y_delta = b_t.translation.y - p_t.translation.y;
    if x_delta < 20.0 && y_delta < 20.0 {
        info!("you're it!");
        p.it = true;
    }
    // find our position in y
    // find bot position in x
    // find bot position in y
    // if delta of both is less than 20, we are tagged

    //let move_speed = MOVE_PER_TICK;
    //let move_delta = direction * move_speed * time.delta_secs();
}
