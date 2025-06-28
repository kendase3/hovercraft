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
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};
use bevy::window::PresentMode;
use bevy::{core_pipeline::bloom::Bloom, prelude::*, text::FontSmoothing};
use bevy::{
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

const MOVE_PER_TICK: f32 = 40.;
const BOT_MOVE_PER_TICK: f32 = 20.;
const PLAYER_RADIUS: f32 = 10.;
const MAP_SIZE: u32 = 400;
const GRID_SIZE: f32 = 1.;
const SPACE_BETWEEN_LINES: u32 = 20;
const CAMERA_DEFAULT_SIZE: f32 = 100.;
// no idea what units this is using, apparently in-game ones, not 0-1
const TARGET_WIDTH: f32 = 2.;
const ORBIT_DISTANCE: f32 = 50.;

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
struct Target;

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
    mut materials2: ResMut<Assets<TargetMaterial>>,
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
            // FIXME(skend): does not render; maybe i need to world.load_asset()
            SceneRoot(
                asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("models/gnat.glb")),
            ),
            Transform {
                translation: Vec3::new(0., 0., 0.),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            //SceneNumber(1),
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
    let bot_target = meshes.add(Mesh::from(Rectangle::new(
        PLAYER_RADIUS * 2.,
        PLAYER_RADIUS * 2.,
    )));
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
    mut bot: Query<&mut Transform, With<Bot>>,
    mut player: Query<&mut Transform, (With<Player>, Without<Bot>)>,
    time: Res<Time>,
) {
    let mut b_t = bot.single_mut();
    let p_t = player.single_mut();

    // receive an x/y coordinate we're currently flying to
    let dest = hovercraft::orbit(
        b_t.translation.xy(),
        p_t.translation.xy(),
        ORBIT_DISTANCE,
    );
    // delta is now between us and our orbit destination
    let move_vector = dest - b_t.translation.xy();

    let move_speed = BOT_MOVE_PER_TICK;
    // make sure to normalize the vector so the speed is correct
    let move_delta = move_vector.normalize() * move_speed * time.delta_secs();
    let old_pos = b_t.translation.xy();
    let limit = Vec2::splat(MAP_SIZE as f32 / 2.);
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
