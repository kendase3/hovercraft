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

use bevy::color::palettes::basic::PURPLE;
use bevy::log::LogPlugin;
use bevy::render::camera::ScalingMode;
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};
use bevy::window::PresentMode;
use bevy::{core_pipeline::bloom::Bloom, prelude::*, text::FontSmoothing};
use bevy::{
    reflect::TypePath,
    render::camera::Exposure,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use hovercraft::Acceleration;
use std::f32::consts::PI;

const BOT_MOVE_PER_TICK: f32 = 20.;
const PLAYER_RADIUS: f32 = 10.;
const GRID_SIZE: f32 = 10.;
const SPACE_BETWEEN_LINES: u32 = 20;
const CAMERA_DEFAULT_SIZE: f32 = 100.;
// no idea what units this is using, apparently in-game ones, not 0-1
const TARGET_WIDTH: f32 = 2.;
const ORBIT_DISTANCE: f32 = 50.;
const ORBIT_CALC_INTERVAL: f32 = 0.2; // in seconds
const MAX_FRAMERATE: f32 = 60.;
const PLANET_COORDS: (f32, f32, f32) = (-50.0, 50.0, 0.0);

#[derive(Component)]
struct Player {
    it: bool,
    facing: f32,
}

#[derive(Component)]
struct Bot {
    it: bool,
}

#[derive(Component)]
struct Proclamation;

#[derive(Component)]
struct Facing;

#[derive(Component)]
struct Target;

#[derive(Component)]
struct TagCooldownTimer {
    timer: Timer,
}

// a planetary body like a planet, asteroid field, a location you can warp to
#[derive(Component)]
struct Warp;

// whether or not the cooldown is ready for a tag to happen
#[derive(Component)]
struct TagReady {
    ready: bool,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TargetMaterial {
    #[uniform(0)] // same
    border_width: f32,
}

impl Material2d for TargetMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/target.wgsl".into()
    }
    // required for transparency
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// rather than perpetually compute the destination, we'll cache it and only check a few times a
// second
#[derive(Resource, Default)]
struct OrbitCache {
    destination: Vec2,
}

#[derive(Resource)]
struct OrbitTimer(Timer);

impl FromWorld for OrbitTimer {
    fn from_world(_: &mut World) -> Self {
        OrbitTimer(Timer::from_seconds(
            ORBIT_CALC_INTERVAL,
            TimerMode::Repeating,
        ))
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // fill whole browser window
                        fit_canvas_to_parent: true,
                        // FIXME(skend): tab still does stuff sadly
                        // there's a javascript hack or i can wait for them to fix it
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
        .add_plugins(Material2dPlugin::<TargetMaterial>::default())
        .insert_resource(ClearColor(Color::srgb(0.53, 0.53, 0.53)))
        .add_systems(Startup, (draw_map, setup))
        .add_systems(
            Update,
            (
                move_player,
                face_all,
                move_bot,
                handle_tag,
                camera_follow,
                handle_target,
            ),
        )
        .init_resource::<OrbitTimer>()
        .init_resource::<OrbitCache>()
        .add_systems(
            FixedUpdate,
            (hovercraft::apply_acceleration, hovercraft::apply_velocity)
                .chain(),
        )
        // FIXME(skend): surely i should name these
        // won't i have dozens of fixed time events eventually?
        .insert_resource(Time::<Fixed>::from_seconds(
            (1.0 / MAX_FRAMERATE).into(),
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut materials2: ResMut<Assets<TargetMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(TagReady { ready: true });
    // create a tag cooldown timer
    commands.spawn(TagCooldownTimer {
        timer: Timer::from_seconds(1.0, TimerMode::Once),
    });
    // FIXME(skend): this light only covers an extremely small area at the center of the map
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 100_000.0,
            color: Color::srgb(1.0, 0.9, 0.9),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 20.0)
            //.with_rotation(Quat::from_rotation_x(0.25 * -PI / 2.)),
            .with_rotation(Quat::from_rotation_x(0.3 * -PI / 2.)),
        // first arg: target, second arg: up
        //.looking_at(Vec3::ZERO, Vec3::Z),
    ));
    /*
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    */
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true, // HDR is required for the bloom effect
            order: 0,
            ..default()
        },
        // 6 was in transmission.rs bevy example
        Exposure { ev100: 10.0 },
        Transform {
            // raise the light above the world so it hits the top faces the viewer sees
            translation: Vec3::new(0., 0., 20.),
            ..default()
        },
        //Bloom::NATURAL,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: CAMERA_DEFAULT_SIZE,
            },
            scale: 1.,
            near: -1000.0,
            far: 1000.0,
            ..OrthographicProjection::default_3d()
        }),
    ));
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
    // load some meshes, colors and fonts used by the player and bot
    // TODO(skend): organize / split this up
    // sin and cos same for 45 case
    let fortyfivepoint = PLAYER_RADIUS * (45.0 as f32).to_radians().sin();
    let player_facing_triangle = meshes.add(Triangle2d::new(
        Vec2::X * PLAYER_RADIUS,
        Vec2::new(-1. * fortyfivepoint, -1. * fortyfivepoint),
        Vec2::new(-1. * fortyfivepoint, fortyfivepoint),
    ));
    let bot_color = Color::srgb(0.0, 0.0, 0.0);
    let player_color = Color::srgb(0.0, 0.0, 0.0);
    let triangle_color = Color::srgb(0.0, 1.0, 1.0);
    let planet_color = Color::srgb(0.0, 1.0, 0.0);
    let player_circle = meshes.add(Circle::new(PLAYER_RADIUS));
    let font = asset_server.load("fonts/DejaVuSansMono.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 100.0,
        ..default()
    };
    commands
        .spawn((
            Player {
                it: false,
                facing: 0.0,
            },
            hovercraft::Velocity(Vec3::new(0., 0., 0.)),
            hovercraft::Acceleration(Vec3::new(0., 0., 0.)),
            Name::new("Protagonist"),
            Mesh2d(player_circle),
            MeshMaterial2d(materials.add(player_color)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("models/gnat2.glb"),
                )),
                // NB(skend): notably does nothing
                Transform {
                    translation: Vec3::new(0., 0., 0.),
                    rotation: Quat::default(),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
                Visibility::Visible,
                Facing,
            ));
            parent.spawn((
                Text2d::new("@"),
                text_font
                    .clone()
                    .with_font_smoothing(FontSmoothing::AntiAliased),
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::srgb(1., 0., 1.)),
                Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.2)),
                Visibility::Visible,
            ));
            parent.spawn((
                Facing,
                Mesh2d(player_facing_triangle),
                MeshMaterial2d(materials.add(triangle_color)),
                Visibility::Visible,
                Transform {
                    translation: Vec3::new(PLAYER_RADIUS * 0.8, 0.0, 0.0),
                    rotation: default(),
                    scale: Vec3::new(0.2, 0.2, 1.0),
                },
            ));
        });
    let bot = meshes.add(Circle::new(PLAYER_RADIUS));
    let bot_target = meshes.add(Mesh::from(Rectangle::new(
        PLAYER_RADIUS * 2.,
        PLAYER_RADIUS * 2.,
    )));
    let planet1 = meshes.add(Circle::new(PLAYER_RADIUS * 2.));
    commands
        .spawn((
            Bot { it: true },
            Name::new("Antagonist"),
            Mesh2d(bot),
            MeshMaterial2d(materials.add(bot_color)),
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
            parent.spawn((
                Mesh2d(bot_target),
                Name::new("Bot Target"),
                Target,
                Visibility::Visible,
                MeshMaterial2d(materials2.add(TargetMaterial {
                    border_width: TARGET_WIDTH,
                })),
                // slightly higher z axis
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });
    // kind of like a notification at the top of the screen
    commands.spawn((
        Text::new("You're gaming!"),
        Proclamation,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        Visibility::Hidden,
    ));
    commands.spawn((
        Warp,
        Name::new("Planet1"),
        Mesh2d(planet1),
        MeshMaterial2d(materials.add(planet_color)),
        Transform::from_xyz(PLANET_COORDS.0, PLANET_COORDS.1, PLANET_COORDS.2),
    ));
}

// generally handle tagging state changes
fn handle_tag(
    mut proclamation: Query<&mut Visibility, With<Proclamation>>,
    mut bot: Query<(&mut Bot, &mut Transform)>,
    mut player: Query<(&mut Player, &mut Transform), Without<Bot>>,
    mut tagready: Query<&mut TagReady>,
    mut tagtimer: Query<&mut TagCooldownTimer>,
    time: Res<Time>,
) {
    let mut timer = tagtimer.single_mut();
    let mut tagr = tagready.single_mut();
    timer.timer.tick(time.delta());
    if timer.timer.finished() {
        tagr.ready = true;
    } else {
        // reduce work this func does when timer not ready
        return;
    }
    let (mut b, b_t) = bot.single_mut();
    let (mut p, p_t) = player.single_mut();
    let x_delta = (b_t.translation.x - p_t.translation.x).abs();
    let y_delta = (b_t.translation.y - p_t.translation.y).abs();
    // if there's a timer that's done, set tagready to ready
    let distance = (x_delta.powf(2.) + y_delta.powf(2.)).sqrt();
    if tagr.ready && distance < 2. * PLAYER_RADIUS {
        info!("you're it!");
        p.it = !p.it;
        b.it = !b.it;
        // begin the cooldown period before we can tag again
        tagr.ready = false;
        // reset our cooldown timer
        timer.timer.reset();
    }

    // update the top text
    let mut proc = proclamation.single_mut();
    if p.it {
        *proc = Visibility::Visible;
    } else {
        *proc = Visibility::Hidden;
    }
}

fn face_all(
    mut facers_query: Query<(&mut Transform, &Parent), With<Facing>>,
    player_query: Query<&Player>,
) {
    for (mut facer, parent) in &mut facers_query {
        if let Ok(player) = player_query.get(parent.get()) {
            facer.rotation = Quat::from_axis_angle(Vec3::Z, player.facing);
        }
    }
}

fn move_player(
    mut players: Query<(&mut Acceleration, &mut Player)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
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

    let (mut accel, mut play) = players.single_mut();
    let n_direction;
    if direction != Vec3::ZERO {
        n_direction = direction.normalize(); // likely unnecessary
    } else {
        n_direction = Vec3::ZERO;
    }
    // the new acceleration value is based on what player is up to
    accel.0 = n_direction * hovercraft::PLAYER_ACCEL_RATE * time.delta_secs();

    // the ship faces whatever input the player last entered
    if direction != Vec3::ZERO {
        play.facing = n_direction.y.atan2(n_direction.x);
    }
}

fn move_bot(
    mut bot: Query<&mut Transform, With<Bot>>,
    mut player: Query<&mut Transform, (With<Player>, Without<Bot>)>,
    time: Res<Time>,
    mut orbit_timer: ResMut<OrbitTimer>,
    mut orbit_cache: ResMut<OrbitCache>,
) {
    // receive an x/y coordinate we're currently flying to
    let mut b_t = bot.single_mut();
    let p_t = player.single_mut();

    orbit_timer.0.tick(time.delta());
    // only update destination if it's time
    if orbit_timer.0.finished() {
        orbit_cache.destination = hovercraft::orbit(
            b_t.translation.xy(),
            p_t.translation.xy(),
            ORBIT_DISTANCE,
        );
    }
    let dest = orbit_cache.destination;

    // delta is now between us and our orbit destination
    let move_vector = dest - b_t.translation.xy();

    let move_speed = BOT_MOVE_PER_TICK;
    // make sure to normalize the vector so the speed is correct
    let move_delta = move_vector.normalize() * move_speed * time.delta_secs();
    let old_pos = b_t.translation.xy();
    let limit = Vec2::splat(hovercraft::MAP_SIZE as f32 / 2.);
    let new_pos = (old_pos + move_delta).clamp(-limit, limit);
    b_t.translation.x = new_pos.x;
    b_t.translation.y = new_pos.y;
}

fn camera_follow(
    playerq: Query<&Transform, With<Player>>,
    botq: Query<&Transform, (With<Bot>, Without<Player>)>,
    mut cameraq: Query<
        &mut Transform,
        (With<Camera>, Without<Player>, Without<Bot>),
    >,
) {
    let ppos = playerq.single().translation;
    let bpos = botq.single().translation;
    let camera_x = (ppos.x + bpos.x) / 2.;
    let camera_y = (ppos.y + bpos.y) / 2.;
    for mut c in &mut cameraq {
        c.translation.x = camera_x;
        c.translation.y = camera_y;
        // see CAMERA_DEFAULT_SIZE for usual camera zoom level
        let radius =
            ((ppos.x - bpos.x).powf(2.) + (ppos.y - bpos.y).powf(2.)).sqrt();
        if radius > CAMERA_DEFAULT_SIZE {
            let zoom_factor = radius / CAMERA_DEFAULT_SIZE;
            c.scale = Vec3::splat(zoom_factor);
        }
    }
}

// TODO(skend): the map should have texture, and maybe a fun fog effect
fn draw_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // we have MAP_SIZE for both width and depth
    for y in ((-1 * hovercraft::MAP_SIZE as i32 / 2)
        ..(hovercraft::MAP_SIZE as i32 / 2))
        .step_by(10)
    {
        for x in ((-1 * hovercraft::MAP_SIZE as i32 / 2)
            ..(hovercraft::MAP_SIZE as i32 / 2))
            .step_by(10)
        {
            let center = Vec3::new(x as f32, y as f32, 0.0);
            let mut matl = |color| {
                materials.add(StandardMaterial {
                    base_color: color,
                    //perceptual_roughness: 1.0,
                    //metallic: 1.0,
                    //emissive: PURPLE.into(),
                    ..default()
                })
            };
            let mut plane = Mesh::from(
                Plane3d {
                    normal: Dir3::Z,
                    half_size: Vec2::new(10., 10.),
                    ..default()
                }
                .mesh(),
            );
            let vertex_colors: Vec<[f32; 4]> = vec![
                LinearRgba::RED.to_f32_array(),
                LinearRgba::GREEN.to_f32_array(),
                LinearRgba::BLUE.to_f32_array(),
                PURPLE.to_f32_array(),
            ];
            plane.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);
            commands.spawn((
                Mesh3d(meshes.add(plane)),
                //MeshMaterial3d(matl(Color::from(PURPLE))),
                MeshMaterial3d(matl(Color::WHITE)),
                Transform::from_xyz(center.x, center.y, -1.0), //.with_scale(Vec3::splat(10. as f32)),
            ));
        }
    }
}

// FIXME(skend): use tab though
// also need a cooldown of like .5 seconds to stop multi-press
fn handle_target(
    keys: Res<ButtonInput<KeyCode>>,
    mut botq: Query<&mut Visibility, With<Target>>,
) {
    let mut b = botq.single_mut();
    if !keys.any_pressed([KeyCode::KeyT]) {
        return;
    }
    if *b == Visibility::Visible {
        *b = Visibility::Hidden;
    } else {
        *b = Visibility::Visible;
    }
}
