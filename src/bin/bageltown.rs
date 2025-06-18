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

use bevy::log::LogPlugin;
use bevy::render::camera::ScalingMode;
use bevy::window::PresentMode;
use bevy::{core_pipeline::bloom::Bloom, prelude::*, text::FontSmoothing};
use bevy::render::mesh::PrimitiveTopology;
use bevy::{reflect::TypePath, render::render_resource::{AsBindGroup, ShaderRef}};

#[derive(Component)]
struct Player {
    it: bool,
}

#[derive(Component)]
struct Bot {
    it: bool,
}

#[derive(Component)]
struct Proclamation;

#[derive(Component)]
struct TagCooldownTimer {
    timer: Timer,
}

// whether or not the cooldown is ready for a tag to happen
#[derive(Component)]
struct TagReady {
    ready: bool,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct TargetMaterial {
    #[uniform(0)] // binding(0) in shader
    color_opaque: Color,
    #[uniform(0)] // binding(0) in shader
    color_transparent: Color,
    #[uniform(0)] // same
    border_width: f32,
}

impl Material for TargetMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/target.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/target.wgsl".into()
    }
    // required for transparency
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

const MOVE_PER_TICK: f32 = 40.;
const BOT_MOVE_PER_TICK: f32 = 20.;
const PLAYER_RADIUS: f32 = 10.;
const TARGET_INNER_OFFSET: f32 = 1.;
const MAP_SIZE: u32 = 400;
const GRID_SIZE: f32 = 1.;
const SPACE_BETWEEN_LINES: u32 = 20;
const CAMERA_DEFAULT_SIZE: f32 = 100.;
const TARGET_WIDTH: f32 = 2.;

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
        .add_systems(Startup, (draw_map, startup))
        .add_systems(
            Update,
            (
                move_player,
                move_bot,
                handle_tag,
                camera_follow,
                handle_target,
            ),
        )
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(TagReady { ready: true });
    // create a tag cooldown timer
    commands.spawn(TagCooldownTimer {
        timer: Timer::from_seconds(1.0, TimerMode::Once),
    });
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true, // HDR is required for the bloom effect
            ..default()
        },
        Bloom::NATURAL,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: CAMERA_DEFAULT_SIZE,
            },
            // This is the default value for scale for orthographic projections.
            // To zoom in and out, change this value, rather than `ScalingMode` or the camera's position.
            scale: 1.,
            ..OrthographicProjection::default_2d()
        }),
    ));
    let player = meshes.add(Circle::new(PLAYER_RADIUS));
    let color = Color::srgb(0.0, 0.0, 0.0);
    let target_color = Color::srgb(1.0, 0.0, 0.0);
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
    let bot = meshes.add(Circle::new(PLAYER_RADIUS));
    //let bot_target = meshes.add(Annulus::new(PLAYER_RADIUS, PLAYER_RADIUS + TARGET_WIDTH));
    let bot_target = meshes.add(Mesh::from(Rectangle::new(PLAYER_RADIUS, PLAYER_RADIUS)));
    commands
        .spawn((
            Bot { it: true },
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
            parent.spawn((
                Mesh2d(bot_target),
                Name::new("Bot Target"),
                //Visibility::Hidden,
                Visibility::Visible,
                //MeshMaterial2d(materials.add(target_color)),
                MeshMaterial2d(materials.add(TargetMaterial { 
                    color_opaque: Color::srgb(1.0, 0.0, 0.0),
                    color_transparent: Color::srgba(0.0, 0.0, 0.0, 0.2),
                    border_width: TARGET_WIDTH}),
                ),
                // slightly higher z axis
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });
    // kind of like a notification at the top of the screen
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

// generally handle tagging state changes
fn handle_tag(
    mut proclamation: Query<&mut Visibility, With<Proclamation>>,
    mut bot: Query<(&mut Bot, &mut Transform)>,
    mut player: Query<(&mut Player, &mut Transform), Without<Bot>>,
    mut tagready: Query<&mut TagReady>,
    mut tagtimer: Query<&mut TagCooldownTimer>,
    time: Res<Time>,
) {
    let (mut b, b_t) = bot.single_mut();
    let (mut p, p_t) = player.single_mut();
    let mut tagr = tagready.single_mut();
    let x_delta = (b_t.translation.x - p_t.translation.x).abs();
    let y_delta = (b_t.translation.y - p_t.translation.y).abs();
    // if there's a timer that's done, set tagready to ready
    let mut timer = tagtimer.single_mut();
    timer.timer.tick(time.delta());
    if timer.timer.finished() {
        tagr.ready = true;
    }
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
    let mut p = players.single_mut();
    let old_pos = p.translation.xy();
    let limit = Vec2::splat(MAP_SIZE as f32 / 2.);
    let new_pos = (old_pos + move_delta).clamp(-limit, limit);

    p.translation.x = new_pos.x;
    p.translation.y = new_pos.y;
}

fn move_bot(
    mut bot: Query<(&mut Bot, &mut Transform)>,
    mut player: Query<(&mut Player, &mut Transform), Without<Bot>>,
    time: Res<Time>,
) {
    let (b, mut b_t) = bot.single_mut();
    let (_, p_t) = player.single_mut();
    let x_delta = b_t.translation.x - p_t.translation.x;
    let y_delta = b_t.translation.y - p_t.translation.y;

    let mut direction = Vec2::ZERO;
    let mut it_multiplier = 1.;
    if b.it {
        it_multiplier = -1.;
    }
    if x_delta < 0. {
        direction.x -= 1.;
    } else if x_delta > 0. {
        direction.x += 1.;
    }
    if y_delta < 0. {
        direction.y -= 1.;
    } else if y_delta > 0. {
        direction.y += 1.;
    }
    // if we're it, run towards player instead of away
    direction *= it_multiplier;

    let move_speed = BOT_MOVE_PER_TICK;
    let move_delta = direction * move_speed * time.delta_secs();
    let old_pos = b_t.translation.xy();
    let limit = Vec2::splat(MAP_SIZE as f32 / 2.);
    let new_pos = (old_pos + move_delta).clamp(-limit, limit);
    b_t.translation.x = new_pos.x;
    b_t.translation.y = new_pos.y;
}

fn camera_follow(
    playerq: Query<(&Player, &Transform)>,
    botq: Query<(&Bot, &Transform), Without<Player>>,
    mut cameraq: Query<
        &mut Transform,
        (With<Camera>, Without<Player>, Without<Bot>),
    >,
) {
    let p = playerq.single();
    let b = botq.single();
    let ppos = p.1.translation;
    let bpos = b.1.translation;
    let camera_x = (ppos.x + bpos.x) / 2.;
    let camera_y = (ppos.y + bpos.y) / 2.;
    let mut c = cameraq.single_mut();
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

fn draw_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // let's make horizontal lines first
    for i in 0..=MAP_SIZE {
        if i % SPACE_BETWEEN_LINES != 0 {
            continue;
        };
        // first we make our line
        let rect_width = MAP_SIZE as f32;
        let rect_height = GRID_SIZE;
        let rect_mesh = meshes.add(Rectangle::new(rect_width, rect_height));
        let rect_color =
            materials.add(ColorMaterial::from(Color::srgb(0., 0., 0.)));

        commands.spawn((
            Mesh2d(rect_mesh),
            MeshMaterial2d(rect_color),
            // we start at negative 1/2 map size, go up to positive 1/2 map size
            Transform::from_xyz(0., i as f32 - MAP_SIZE as f32 / 2., 0.),
        ));
    }
    // then vertical
    for i in 0..=MAP_SIZE {
        if i % SPACE_BETWEEN_LINES != 0 {
            continue;
        };
        // first we make our line
        let rect_width = GRID_SIZE;
        let rect_height = MAP_SIZE as f32;
        let rect_mesh = meshes.add(Rectangle::new(rect_width, rect_height));
        let rect_color =
            materials.add(ColorMaterial::from(Color::srgb(0., 0., 0.)));

        commands.spawn((
            Mesh2d(rect_mesh),
            MeshMaterial2d(rect_color),
            // we start at negative 1/2 map size, go up to positive 1/2 map size
            Transform::from_xyz(i as f32 - MAP_SIZE as f32 / 2., 0., 0.),
        ));
    }
}

// TODO(skend): tab targeting
// for now just render a target around the bot
// maybe there's always a target around each targetable entity
// but it's just toggled to hidden
fn handle_target() {
    // so the bot has a certain radius already.
}
