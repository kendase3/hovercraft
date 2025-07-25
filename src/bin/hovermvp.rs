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

use hovercraft::laser;
use hovercraft::physics;

use bevy::animation::{AnimationClip, AnimationPlayer};
use bevy::audio::Volume;
use bevy::color::palettes::basic::PURPLE;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::log::LogPlugin;
use bevy::render::camera::ScalingMode;
use bevy::render::mesh::{Indices, Mesh};
use bevy::scene::SceneInstanceReady;
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};
use bevy::window::PresentMode;
use bevy::{core_pipeline::bloom::Bloom, prelude::*, text::FontSmoothing};
use bevy::{
    reflect::TypePath,
    render::camera::Exposure,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use physics::Acceleration;
use rand::Rng;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::time::Duration;

const BOT_RADIUS: f32 = 10.;
const PLANET_RADIUS: f32 = 15.;
const GRID_SIZE: f32 = 10.;
// TODO(skend): chunks
//const CHUNK_SIZE: f32 = GRID_SIZE * 2.;
const CAMERA_DEFAULT_SIZE: f32 = 100.;
// no idea what units this is using, apparently in-game ones, not 0-1
const TARGET_WIDTH: f32 = 2.;
const ORBIT_DISTANCE: f32 = 50.;
const ORBIT_CALC_INTERVAL: f32 = 0.2; // in seconds
const MAX_FRAMERATE: f32 = 60.;
const PLANET_COORDS: (f32, f32, f32) = (-50.0, 50.0, 0.0);
// TODO(skend): simplify these and they do not have to be constants
// half of them like "shrinker" are just inline for some reason
const NOTCH_OUTER_SIZE: f32 = 5.;
const NOTCH_INNER_SIZE: f32 = 4.75;
const NOTCH_TRIANGLE_RADIUS_KINDOF: f32 = 20.;
const BOT_START_OFFSET: f32 = 50.;
const GNAT_EXPLODE_PATH: &str = "models/gnat2explosion.glb";
const GUBBINS_EXPLODE_PATH: &str = "models/gubbins2explosion.glb";
const GNAT_PATH: &str = "models/gnat2_6.glb";
const GUBBINS_PATH: &str = "models/gubbins2.glb";
// number of tile variants for the plaidsea
const NUM_TILES: u32 = 10;
const BOT_LASER_INTERVAL_SECONDS: u64 = 5;

#[derive(Component, PartialEq, Debug)]
enum PilotType {
    Player,
    Bot,
}

impl Default for PilotType {
    fn default() -> Self {
        PilotType::Bot
    }
}

#[derive(Component, Default, PartialEq, Debug)]
struct Pilot {
    pilottype: PilotType,
    it: bool,
    // FIXME(skend): suppose bots do not have facing
    // as is the case right now. how would we use the
    // enum to handle this split?
    facing: Option<f32>,
    target: Option<Entity>,
    needs_start_fire_large_laser: bool,
    still_firing_large_laser: bool,
    laser_timer: Option<Timer>,
    // TODO(skend): eventually i can turn this into a dict
    // of weapons, where looking up large laser returns
    // both the entity of the cannon and the laser itself.
    // For now let's proof of concept it more simply.
    cannon: Option<Entity>,
    laser: Option<Entity>,
    ship: Option<Entity>,
    // TODO(skend): make an enum
    dead: bool,
    just_died: bool,
    lookup: Option<Entity>,
}

// is it actually fine to not have normal form
// if it makes lookups faster? now i have
// learned how to fix later if i must
#[derive(Component)]
struct Player;
#[derive(Component)]
struct Bot;

// a dude is the union between a player and a bot
// a player-like entity
#[derive(Component)]
struct DudeRef(Entity);

// like dudes but for ships
#[derive(Component)]
struct Craft(Entity);

#[derive(Component)]
struct Proclamation;

#[derive(Component)]
struct Facing;

// FIXME(skend): this is just for the GUI element to target things currently
// what doesn't map very well is that bots will also have targets. there
// won't be a GUI indicator but we will still need to do target lookup
// for aiming etc.
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

// the glb asset itself
#[derive(Component)]
struct ShipModel;

#[derive(Component)]
struct ExplodingModel {
    timer: Timer,
    duration: f32,
    exploded: bool,
}

impl Default for ExplodingModel {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(3., TimerMode::Once),
            duration: 3.,
            exploded: false,
        }
    }
}

// we have to add this component after initial load
// because it's a child in the glb
// FIXME(skend): should just call cannon and ship? it's not like there's a non-model version of
// these
#[derive(Component)]
struct CannonModel;

#[derive(Component)]
struct NotCannonModel;

#[derive(Component)]
struct NotchOffset(pub Vec3);

#[derive(Component)]
struct LargeLaser;

#[derive(Resource, Default)]
struct LaserSound {
    pub sound: Handle<AudioSource>,
    // each pilot can play their own sound
    // and this dict tracks that
    pub is_playing: HashMap<Entity, bool>,
}

#[derive(Resource, Default)]
struct CannonInitialized(bool);

#[derive(Resource, Default)]
struct LaserInitialized(bool);

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

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LaserMaterial {}

impl Material for LaserMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/animate.wgsl".into()
    }
    // required for transparency
    //fn alpha_mode(&self) -> AlphaMode {
    //    AlphaMode::Blend
    //}
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ExplodeMaterial {
    #[uniform(0)]
    explode_center: Vec3,
    #[uniform(1)]
    explode_progress: f32,
}

impl Material for ExplodeMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/explode.wgsl".into()
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

#[derive(Resource)]
struct Timer10hzForBot(Timer);
#[derive(Resource)]
struct Timer10hzForLaser(Timer);
#[derive(Resource)]
struct Timer1hz(Timer);
#[derive(Resource)]
struct Timer30sForBotLaser(Timer);

impl FromWorld for OrbitTimer {
    fn from_world(_: &mut World) -> Self {
        OrbitTimer(Timer::from_seconds(
            ORBIT_CALC_INTERVAL,
            TimerMode::Repeating,
        ))
    }
}

// FIXME(skend): surely there's a way to just have one of these
fn dont_need_cannon_init(ci: Res<CannonInitialized>) -> bool {
    ci.0
}

fn need_cannon_init(ci: Res<CannonInitialized>) -> bool {
    !ci.0
}

fn dont_need_laser_init(ci: Res<LaserInitialized>) -> bool {
    ci.0
}

fn need_laser_init(ci: Res<LaserInitialized>) -> bool {
    !ci.0
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
        .add_plugins(MaterialPlugin::<LaserMaterial>::default())
        .add_plugins(MaterialPlugin::<ExplodeMaterial>::default())
        //.add_plugins(FrameTimeDiagnosticsPlugin::default())
        //.add_plugins(LogDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::srgb(0.53, 0.53, 0.53)))
        .insert_resource(CannonInitialized(false))
        .insert_resource(LaserInitialized(false))
        .add_systems(Startup, (init_plaidsea, (setup, init_targets).chain()))
        .add_systems(PreUpdate, (init_ship).run_if(need_cannon_init))
        .add_systems(PreUpdate, (init_laser).run_if(need_laser_init))
        .add_systems(
            Update,
            (
                move_player,
                face_all,
                rotface_all,
                handle_tag,
                camera_follow,
                handle_target,
                explode,
            ),
        )
        .add_systems(Update, (aim_cannon).run_if(dont_need_cannon_init))
        .add_systems(FixedUpdate, (handle_laser).run_if(dont_need_laser_init))
        .init_resource::<OrbitTimer>()
        .init_resource::<OrbitCache>()
        // TODO(skend): not used for anything yet
        .insert_resource(Timer1hz(Timer::new(
            Duration::from_secs(1),
            TimerMode::Repeating,
        )))
        .insert_resource(Timer10hzForBot(Timer::new(
            Duration::from_millis(100),
            TimerMode::Repeating,
        )))
        .insert_resource(Timer10hzForLaser(Timer::new(
            Duration::from_millis(100),
            TimerMode::Repeating,
        )))
        .insert_resource(Timer30sForBotLaser(Timer::new(
            Duration::from_secs(BOT_LASER_INTERVAL_SECONDS),
            TimerMode::Repeating,
        )))
        .add_systems(
            FixedUpdate,
            (
                (physics::apply_acceleration, physics::apply_velocity).chain(),
                move_bot,
            ),
        )
        .insert_resource(Time::<Fixed>::from_seconds(
            (1.0 / MAX_FRAMERATE).into(),
        ))
        .run();
}

fn explode(
    time: Res<Time>,
    mut query: Query<&mut ExplodingModel>,
    mut materials: ResMut<Assets<ExplodeMaterial>>,
) {
}

fn init_laser(
    qlaserstuff: Query<(Entity, &Parent), With<LargeLaser>>,
    mut pilot_query: Query<&mut Pilot>,
    q_name: Query<&Name>,
    mut commands: Commands,
    mut laser_initialized: ResMut<LaserInitialized>,
) {
    for (laser_entity, laser_parent) in qlaserstuff.iter() {
        let name = q_name.get(laser_entity);
        if let Ok(name_success) = name {
            //info!("cur name = {}", name_success);
            // TODO(skend): i could index a ship's cannons
            // i.e. laser0 looks for cannon0
            // and then we map them one time in init_laser
            if name_success.as_str() == "laser" {
                // first we'll set up the laser link from the pilot
                let mut pilot =
                    pilot_query.get_mut(laser_parent.get()).unwrap();
                pilot.laser = Some(laser_entity);
                // then we'll go about setting up the laser itself
                if let Some(mut entity_commands) =
                    commands.get_entity(laser_entity)
                {
                    entity_commands.insert(DudeRef(laser_parent.get()));
                    info!("initialized a laser");
                }
            }
        }
    }
    // TODO(skend): here we can reparent the laser s.t. the cannon is its parent

    laser_initialized.0 = true;
}

fn init_ship(
    ship_stuff: Query<(Entity, &Parent), With<ShipModel>>,
    children: Query<&Children>,
    mut pilot_query: Query<&mut Pilot>,
    q_name: Query<&Name>,
    mut commands: Commands,
    mut cannon_initialized: ResMut<CannonInitialized>,
) {
    for (ship_gubbins, ship_parent) in &ship_stuff {
        for entity in children.iter_descendants(ship_gubbins) {
            let name = q_name.get(entity);
            if let Ok(name_success) = name {
                if name_success.as_str() == "cannon" {
                    // first update the pilot's cannon link
                    let mut pilot =
                        pilot_query.get_mut(ship_parent.get()).unwrap();
                    pilot.cannon = Some(entity);
                    pilot.ship = Some(ship_gubbins);

                    // then do the work for the cannon itself
                    if let Some(mut entity_commands) =
                        commands.get_entity(entity)
                    {
                        entity_commands.insert(CannonModel {});
                        entity_commands.insert(Visibility::Inherited);
                        entity_commands.insert(DudeRef(ship_parent.get()));
                        entity_commands.insert(Craft(ship_gubbins));
                        if let Ok(pilot) = pilot_query.get(ship_parent.get()) {
                            // then the ship's parent is a player
                            cannon_initialized.0 = true;
                        }
                    }
                } else {
                    // don't actually need this
                    if let Some(mut entity_commands) =
                        commands.get_entity(entity)
                    {
                        entity_commands.insert(NotCannonModel {});
                    }
                }
            }
        }
    }
}
// temp
#[derive(Component)]
struct AnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
    ready: bool,
}

// does not play animation, but does setup once
// animation is loaded
fn mark_animation_ready(
    trigger: Trigger<SceneInstanceReady>,
    mut animations_to_play: Query<&mut AnimationToPlay>,
    mut commands: Commands,
    children: Query<&Children>,
    qwiggler: Query<&AnimationPlayer>,
) {
    if let Ok(mut animation_to_play) =
        animations_to_play.get_mut(trigger.entity())
    {
        animation_to_play.ready = true;
    }
    if let Ok(animation_to_play) = animations_to_play.get(trigger.entity()) {
        for child in children.iter_descendants(trigger.entity()) {
            if let Ok(_) = qwiggler.get(child) {
                commands.entity(child).insert(AnimationGraphHandle(
                    animation_to_play.graph_handle.clone(),
                ));
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut materials2: ResMut<Assets<TargetMaterial>>,
    // will likely use soon
    //mut materials3: ResMut<Assets<StandardMaterial>>,
    mut materials4: ResMut<Assets<LaserMaterial>>,
    mut materials5: ResMut<Assets<ExplodeMaterial>>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // audio imports
    let bloo_sound = asset_server.load("sounds/laser.ogg");
    commands.insert_resource(LaserSound {
        sound: bloo_sound,
        ..default()
    });
    // would not animations also be fun?
    let (graph, animation_index) =
        AnimationGraph::from_clip(asset_server.load(
            GltfAssetLabel::Animation(0).from_asset(GUBBINS_EXPLODE_PATH),
        ));
    let graph_handle = graphs.add(graph);
    let animation_to_play = AnimationToPlay {
        graph_handle,
        index: animation_index,
        ready: false,
    };
    let animation_scene = SceneRoot(
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(GUBBINS_EXPLODE_PATH)),
    );
    // used later in bot
    commands.spawn(TagReady { ready: true });
    // create a tag cooldown timer
    commands.spawn(TagCooldownTimer {
        timer: Timer::from_seconds(1.0, TimerMode::Once),
    });
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 100_000.0,
            color: Color::srgb(1.0, 0.9, 0.9),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 20.0)
            .with_rotation(Quat::from_rotation_x(0.3 * -PI / 2.)),
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
            // hdr required for bloom and also apparently to display us at all
            hdr: true,
            order: 0,
            ..default()
        },
        // lower is brighter
        Exposure { ev100: 12.0 },
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
    let kewlangle = 30.;
    let shrinker = 0.15;
    let triangle_sin =
        NOTCH_TRIANGLE_RADIUS_KINDOF * (kewlangle as f32).to_radians().sin();
    let triangle_cos =
        NOTCH_TRIANGLE_RADIUS_KINDOF * (kewlangle as f32).to_radians().cos();
    // TODO(skend): just do vector math instead of doing this 3 times
    let player_facing_triangle = meshes.add(Triangle2d::new(
        (Vec2::X * NOTCH_TRIANGLE_RADIUS_KINDOF) * shrinker,
        (Vec2::new(-1. * triangle_sin, -1. * triangle_cos)) * shrinker,
        (Vec2::new(-1. * triangle_sin, triangle_cos)) * shrinker,
    ));
    let triangle_color = Color::srgb(0.0, 1.0, 1.0);
    let planet_color = Color::srgb(0.0, 1.0, 0.0);
    //let lol: Handle<Image> = asset_server.load("textures/lol.png");
    let font = asset_server.load("fonts/DejaVuSansMono.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 100.0,
        ..default()
    };
    let notch_circle =
        meshes.add(Annulus::new(NOTCH_INNER_SIZE, NOTCH_OUTER_SIZE));
    let laser_mesh = Cuboid::new(1.0, 1.0, 1.0);
    let notch_offset = Vec3::new(NOTCH_OUTER_SIZE, 0., 0.);
    let kewl_material = materials4.add(LaserMaterial {});
    let explode_material = materials5.add(ExplodeMaterial {
            explode_center: Vec3::ZERO,
            explode_progress: 0.,
    });
    commands
        .spawn((
            Pilot {
                pilottype: PilotType::Player,
                ..default()
            },
            Player,
            physics::Velocity(
                Vec3::new(0., 0., 0.),
                physics::PLAYER_MAX_VELOCITY,
            ),
            physics::Acceleration(
                Vec3::new(0., 0., 0.),
                physics::PLAYER_ACCEL_RATE,
            ),
            Name::new("Protagonist"),
            Transform::default(),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(
                    asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset(GNAT_PATH)),
                ),
                // NB(skend): notably does nothing
                Transform {
                    translation: Vec3::new(0., 0., 0.),
                    rotation: Quat::default(),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
                Visibility::Visible,
                Facing,
                ShipModel,
            ));
            parent.spawn((
                Text2d::new("@"),
                text_font
                    .clone()
                    .with_font_smoothing(FontSmoothing::AntiAliased),
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::srgb(1., 0., 1.)),
                Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.2)),
                Visibility::Hidden,
            ));
            parent.spawn((
                Mesh2d(player_facing_triangle),
                MeshMaterial2d(materials.add(triangle_color)),
                Visibility::Visible,
                Transform {
                    translation: notch_offset,
                    rotation: default(),
                    scale: Vec3::new(0.2, 0.2, 1.0),
                },
                NotchOffset(notch_offset),
            ));
            parent.spawn((
                Mesh2d(notch_circle),
                MeshMaterial2d(materials.add(triangle_color)),
                Visibility::Visible,
            ));
            // TODO(skend): i think this actually should be a child on the cannon.
            // so spawning it would be a little weird/late
            // seems like i may want an initial loading screen
            // See extreme bevy's loading state as an example
            parent.spawn((
                Mesh3d(meshes.add(laser_mesh)),
                MeshMaterial3d(kewl_material.clone()),
                Visibility::Hidden,
                LargeLaser,
                Name::new("laser"),
            ));
        });
    let bot_target = meshes
        .add(Mesh::from(Rectangle::new(BOT_RADIUS * 2., BOT_RADIUS * 2.)));
    let planet1 = meshes.add(Circle::new(PLANET_RADIUS * 2.));
    commands
        .spawn((
            Pilot {
                pilottype: PilotType::Bot,
                it: true,
                ..default()
            },
            Bot,
            Name::new("Antagonist"),
            Transform::from_xyz(BOT_START_OFFSET, 0., 0.),
            Visibility::Hidden,
            physics::Velocity(
                Vec3::new(0., 0., 0.),
                physics::BOT_MAX_VELOCITY,
            ),
            physics::Acceleration(
                Vec3::new(0., 0., 0.),
                physics::BOT_ACCEL_RATE,
            ),
        )).insert(ExplodingModel::default())
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(
                    asset_server.load(
                        GltfAssetLabel::Scene(0).from_asset(GUBBINS_PATH),
                    ),
                ),
                // NB(skend): notably does nothing
                Transform {
                    translation: Vec3::new(0., 0., 0.),
                    rotation: Quat::default(),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
                Visibility::Visible,
                Facing,
                ShipModel,
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
            parent
                .spawn((
                    animation_to_play,
                    animation_scene,
                    Visibility::Hidden,
                ))
                .observe(mark_animation_ready);
            parent.spawn((
                Mesh3d(meshes.add(laser_mesh)),
                MeshMaterial3d(kewl_material.clone()),
                Visibility::Hidden,
                LargeLaser,
                Name::new("laser"),
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

fn init_targets(mut query: Query<(Entity, &mut Pilot)>) {
    let mut player_id: Option<Entity> = None;
    let mut bot_id: Option<Entity> = None;
    for (entity, pilot) in query.iter() {
        if pilot.pilottype == PilotType::Player {
            player_id = Some(entity);
        } else if pilot.pilottype == PilotType::Bot {
            bot_id = Some(entity);
        }
    }
    for (entity, mut pilot) in query.iter_mut() {
        // we also set the entity field on the pilot itself
        pilot.lookup = Some(entity);
        if pilot.pilottype == PilotType::Player {
            pilot.target = bot_id;
        } else if pilot.pilottype == PilotType::Bot {
            pilot.target = player_id;
        }
    }
}

fn handle_laser(
    mut qpilot: Query<&mut Pilot>,
    qtransform: Query<&mut Transform>,
    qentity: Query<Entity, With<Pilot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    qlasermesh: Query<&Mesh3d, With<LargeLaser>>,
    mut qlaservisibility: Query<&mut Visibility, With<LargeLaser>>,
    mut commands: Commands,
    mut laser_sound: ResMut<LaserSound>,
    mut timer: ResMut<Timer10hzForLaser>,
    time: Res<Time>,
) {
    // supposedly you can call time.delta() multiple times in a func
    // but just reducing variables
    let time_delta = time.delta();
    // we can iterate over all the pilots and see if their timers are up
    // make the laser invisible, set the bools appropriately
    // TODO(skend): currently we check this very often to ensure
    // the laser ends at the correct time.
    // It's likely we do not need to check it quite so often
    // and can consolidate the time here to also use fixed time
    // rather than confusingly having both in the func
    for mut pilot in qpilot.iter_mut() {
        let mut finished = false;
        if let Some(lt) = pilot.laser_timer.as_mut() {
            lt.tick(time_delta);
            if lt.finished() {
                finished = true;
            }
        }
        if finished {
            // get our laser and hide it
            let mut finally_laser_time =
                qlaservisibility.get_mut(pilot.laser.unwrap()).unwrap();
            *finally_laser_time = Visibility::Hidden;
            pilot.still_firing_large_laser = false;
            let pilot_entity = pilot.lookup.unwrap();
            laser_sound.is_playing.insert(pilot_entity, false);
        }
    }
    timer.0.tick(time_delta);
    if !timer.0.finished() {
        // this func only fires every 10hz
        return;
    }
    let mut pilots_to_mark: Vec<Entity> = Vec::new();
    let mut pilots_to_kill: Vec<Entity> = Vec::new();
    for pilot in qpilot.iter() {
        let mut laser_origin: Option<Vec2> = None;
        let mut laser_dest: Option<Vec2> = None;
        if pilot.needs_start_fire_large_laser || pilot.still_firing_large_laser
        {
            // so we'll find our target
            if let Some(target) = pilot.target {
                if let Ok(target_transform) = qtransform.get(target) {
                    laser_dest = Some(target_transform.translation.xy());
                    for entity in qentity.iter() {
                        if *qpilot.get(entity).unwrap() == *pilot {
                            // we will do special logic later if we just started firing
                            // we do it later for borrowing reasons
                            // sorry : (
                            if pilot.needs_start_fire_large_laser {
                                pilots_to_mark.push(entity);
                            }
                            if let Ok(pilot_transform) = qtransform.get(entity)
                            {
                                let ship_transform = qtransform
                                    .get(pilot.ship.unwrap())
                                    .unwrap();
                                let cannon_transform = qtransform
                                    .get(pilot.cannon.unwrap())
                                    .unwrap();
                                laser_origin = Some(
                                    pilot_transform.translation.xy()
                                        + ship_transform.translation.xy()
                                        + cannon_transform.translation.xy(),
                                );
                            }
                        }
                    }
                }
            }
            let mut real_laser_origin = laser_origin.unwrap().clone();
            real_laser_origin.x = 0.;
            real_laser_origin.y = 0.;
            // rework origin and dest relative to idea that origin is 0.0
            // and dest is now relative to 0.0
            let real_laser_dest = laser_dest.unwrap() - laser_origin.unwrap();

            let target_is_dead =
                laser::hits(real_laser_origin, real_laser_dest);
            // only do this when the laser starts firing
            if pilot.needs_start_fire_large_laser && target_is_dead {
                if let Some(target) = pilot.target {
                    pilots_to_kill.push(target);
                }
            }

            // NB(skend): no unwrap for this. technically a user could hit it early
            // before these are assigned.
            if let Some(notmeshyet) = pilot.laser {
                if let Ok(mesh) = qlasermesh.get(notmeshyet) {
                    // this unwrap almost certainly fine
                    let actual_mesh = meshes.get_mut(mesh).unwrap();
                    let (laser_vertices, laser_indices, laser_uvs) =
                        laser::get_laser_vertices(
                            real_laser_origin,
                            real_laser_dest,
                        );
                    actual_mesh.insert_attribute(
                        Mesh::ATTRIBUTE_POSITION,
                        laser_vertices,
                    );
                    actual_mesh.insert_indices(Indices::U32(laser_indices));
                    actual_mesh
                        .insert_attribute(Mesh::ATTRIBUTE_UV_0, laser_uvs);
                    // need to fire more than once?
                    //actual_mesh.compute_smooth_normals();
                    actual_mesh.duplicate_vertices();
                    actual_mesh.compute_flat_normals();
                    let mut finally_laser_time = qlaservisibility
                        .get_mut(pilot.laser.unwrap())
                        .unwrap();
                    *finally_laser_time = Visibility::Visible;
                    // FIXME(skend): unify this with the visual aspect in pilot
                    let pilot_entity = pilot.lookup.unwrap();
                    let is_playing = laser_sound.is_playing.get(&pilot_entity);
                    if is_playing.is_none()
                        || !laser_sound.is_playing.get(&pilot_entity).unwrap()
                    {
                        commands.spawn(AudioBundle {
                            source: AudioPlayer(laser_sound.sound.clone()),
                            settings: PlaybackSettings::ONCE
                                .with_volume(Volume::new(0.5)),
                        });
                        laser_sound.is_playing.insert(pilot_entity, true);
                    }
                }
            }
        }
    }
    // well that's one way to dodge the double-borrow
    for pilot_entity in pilots_to_mark.iter() {
        let cur_pilot = qpilot.get_mut(*pilot_entity);
        let mut p = cur_pilot.unwrap();
        p.needs_start_fire_large_laser = false;
        p.still_firing_large_laser = true;
        // we need to reset our laser timer
        // FIXME(skend): this should also have an
        // index to look up which laser to work on
        if p.laser_timer.is_none() {
            p.laser_timer = Some(Timer::from_seconds(
                laser::LASER_DURATION,
                TimerMode::Once,
            ));
        } else {
            p.laser_timer.as_mut().unwrap().reset();
        }
    }
    for pilot_entity in pilots_to_kill.iter() {
        let mut p = qpilot.get_mut(*pilot_entity).unwrap();
        p.dead = true;
        // TODO(skend): make livingness an enum
        p.just_died = true;
    }
}

// TODO(skend): this will actually become the collision/draw logic
// see i knew there was a good reason i kept it for like two weeks
fn handle_tag(
    mut proclamation: Query<&mut Visibility, With<Proclamation>>,
    mut bot: Query<(&mut Pilot, &mut Transform), (With<Bot>, Without<Player>)>,
    mut player: Query<
        (&mut Pilot, &mut Transform),
        (With<Player>, Without<Bot>),
    >,
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
    // FIXME(skend): player does not really have an obvious radius anymore
    if tagr.ready && distance < 2. * BOT_RADIUS {
        info!("you're gaming!");
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
    pilot_query: Query<&Pilot>,
) {
    for (mut facer, parent) in &mut facers_query {
        if let Ok(player) = pilot_query.get(parent.get()) {
            if let Some(player_facing) = player.facing {
                facer.rotation = Quat::from_axis_angle(Vec3::Z, player_facing);
            }
        }
    }
}

fn rotface_all(
    mut facers_query: Query<
        (&mut Transform, &Parent, &NotchOffset),
        With<NotchOffset>,
    >,
    player_query: Query<&Pilot, With<Player>>,
) {
    for (mut facer, parent, offset) in &mut facers_query {
        if let Ok(player) = player_query.get(parent.get()) {
            // we apply our intended offset from spawn to our new relative angle
            if let Some(player_facing) = player.facing {
                facer.rotation = Quat::from_axis_angle(Vec3::Z, player_facing);
                facer.translation = facer.rotation * offset.0;
            }
        }
    }
}

fn aim_cannon(
    mut cannon: Query<(&mut Transform, &DudeRef, &Craft), With<CannonModel>>,
    pilots: Query<&Pilot>,
    qtransform: Query<&Transform, Without<CannonModel>>,
) {
    for (mut cannon_transform, dude, craft) in cannon.iter_mut() {
        let our_cannon_xy = cannon_transform.translation.xy();
        let mut our_ship_xy: Option<Vec2> = None;
        let mut our_dude_xy: Option<Vec2> = None;
        let mut target_xy: Option<Vec2> = None;
        let ship_transform = qtransform.get(craft.0).unwrap();
        if let Ok(cur_pilot) = pilots.get(dude.0) {
            if let Some(cur_target) = cur_pilot.target {
                if let Ok(their_pilot_t) = qtransform.get(cur_target) {
                    target_xy = Some(their_pilot_t.translation.xy());
                }
            }
            if let Ok(pilot_t) = qtransform.get(dude.0) {
                our_dude_xy = Some(pilot_t.translation.xy());
            }
        }
        if let Ok(craft_t) = qtransform.get(craft.0) {
            our_ship_xy = Some(craft_t.translation.xy());
        }

        if our_ship_xy == None || our_dude_xy == None || target_xy == None {
            warn!("Error in aim function!");
            return;
        }
        let delta_loc = target_xy.unwrap() - our_dude_xy.unwrap()
            + our_ship_xy.unwrap()
            + our_cannon_xy;
        let radians = delta_loc.y.atan2(delta_loc.x);
        //info!("the angle in degrees is {}", radians * (180. / PI));
        cannon_transform.rotation =
            Quat::from_rotation_z(radians) * ship_transform.rotation.inverse();
    }
}

fn move_player(
    mut players: Query<(&mut Acceleration, &mut Pilot), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (mut accel, mut play) = players.single_mut();
    // FIXME(skend): complete rework
    // W now accelerates forward in the current direction
    // A and D now modify theta's derivative, accelerating angularly that way
    // and S now accelerates backward.
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
    if keys.any_pressed([KeyCode::KeyR]) {
        if !play.still_firing_large_laser {
            play.needs_start_fire_large_laser = true;
        }
    }

    let n_direction;
    if direction != Vec3::ZERO {
        n_direction = direction.normalize(); // likely unnecessary
    } else {
        n_direction = Vec3::ZERO;
    }
    // the new acceleration value is based on what player is up to
    // accel.1 = acceleration rate
    accel.0 = n_direction * accel.1 * time.delta_secs();

    // the ship faces whatever input the player last entered
    if direction != Vec3::ZERO {
        play.facing = Some(n_direction.y.atan2(n_direction.x));
    }
}

// FIXME(skend): on a lot of these initial functions i made i didn't understand
// how to use get with outer classes to look up inner classes. as such i have
// quite a few particularly heinous, large, disjoint query sets like this.
// i should rework this stuff to have simple queries: one for transforms, one for visibilities,
// etc.
// Then I could just use get() with the pointers i'll have stored from the outer classes
fn move_bot(
    mut bot: Query<
        (
            &mut Transform,
            &mut Pilot,
            &mut physics::Velocity,
            &mut physics::Acceleration,
        ),
        (With<Bot>, Without<Player>, Without<CannonModel>),
    >,
    mut qshipvis: Query<&mut Visibility, With<ShipModel>>,
    qshipt: Query<
        &Transform,
        (
            With<ShipModel>,
            Without<AnimationToPlay>,
            Without<Player>,
            Without<Bot>,
        ),
    >,
    mut qexplosionvis: Query<
        &mut Visibility,
        (With<AnimationToPlay>, Without<ShipModel>, Without<Bot>),
    >,
    mut qexplosiont: Query<
        &mut Transform,
        (With<AnimationToPlay>, Without<ShipModel>, Without<Bot>),
    >,
    mut player: Query<
        &mut Transform,
        (With<Player>, Without<Bot>, Without<AnimationToPlay>),
    >,
    mut qanimation: Query<&mut AnimationToPlay>,
    mut qwiggler: Query<&mut AnimationPlayer>,
    mut orbit_timer: ResMut<OrbitTimer>,
    mut orbit_cache: ResMut<OrbitCache>,
    mut timer: ResMut<Timer10hzForBot>,
    mut timer_for_laser: ResMut<Timer30sForBotLaser>,
    time: Res<Time<Fixed>>,
) {
    let time_delta = time.delta();
    timer_for_laser.0.tick(time_delta);
    let mut bot_laser_timer_finished = false;
    if timer_for_laser.0.finished() {
        bot_laser_timer_finished = true;
    }
    timer.0.tick(time_delta);
    if !timer.0.finished() && !bot_laser_timer_finished {
        // this func only fires every 10hz
        return;
    }
    // receive an x/y coordinate we're currently flying to
    let (b_t, mut b_p, b_v, mut b_a) = bot.single_mut();
    if b_p.dead {
        if b_p.just_died {
            b_p.just_died = false;
            // run special logic like hide the default model
            // FIXME(skend): weirdly this caused a crash for me just now
            // there was no ship visibility? that doesn't make much sense to me
            let mut ship_vis = qshipvis.get_mut(b_p.ship.unwrap()).unwrap();
            *ship_vis = Visibility::Hidden;
            // run special logic to reveal the exploding ship model
            // TODO(skend): give the pilot a ref to its exploding model for lookup
            // right now there's only one so we'll shortcut it.
            let shipt = qshipt.get(b_p.ship.unwrap()).unwrap();
            let mut explode_t = qexplosiont.single_mut();
            explode_t.rotation = shipt.rotation;
            let mut explode_vis = qexplosionvis.single_mut();
            *explode_vis = Visibility::Visible;

            // run special logic to begin the animation
            // TODO(skend): eventually i will have more than one of these and will have to look up
            // the right one
            let anim = qanimation.single_mut();
            //let mut wiggler = wigglers.get_mut(anim.
            //let mut wiggler = qwiggler.single_mut();
            //wiggler.play(anim.index).repeat();
            for mut wiggler in qwiggler.iter_mut() {
                // what if we just blast all the animations
                // disconcertingly, that did not work.
                //wiggler.play(anim.index).repeat();
                wiggler.play(anim.index);
            }

            // run special logic to start a timer to also then mark the exploded ship invisible
        }
        // it's dead so it does not do the normal stuff
        return;
    }
    // FIXME(skend): use bot target instead of single_mut()
    let p_t = player.single_mut();
    // logic to handle bot firing its laser at you
    if bot_laser_timer_finished {
        if !b_p.still_firing_large_laser {
            // notably the bot's target was
            // already set to be the player
            b_p.needs_start_fire_large_laser = true;
        }
    }

    orbit_timer.0.tick(time_delta);
    // only update destination if it's time
    if orbit_timer.0.finished() {
        orbit_cache.destination = physics::orbit(
            b_t.translation.xy(),
            p_t.translation.xy(),
            ORBIT_DISTANCE,
        );
    }
    let dest = orbit_cache.destination;
    // delta is now between us and our orbit destination
    // NB(skend): this is more like our desired move vector
    let desired_move_vector = dest - b_t.translation.xy();
    // NB(skend): we actually need our velocity too, since how much we gas up depends
    // on our current velocity
    let accel_direction = (desired_move_vector - b_v.0.xy()).normalize();
    b_a.0 = accel_direction.extend(0.);

    // update our facing
    b_p.facing = Some(accel_direction.y.atan2(accel_direction.x));
}

fn camera_follow(
    playerq: Query<&Transform, (With<Player>, Without<Bot>)>,
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

fn get_random_color() -> LinearRgba {
    let mut rng = rand::rng();
    let rand_red = rng.random_range(0.0..=0.5) as f32;
    let rand_green = rng.random_range(0.0..=0.5) as f32;
    let rand_blue = rng.random_range(0.0..=0.5) as f32;
    LinearRgba::new(rand_red, rand_green, rand_blue, 1.0)
}

// TODO(skend): in order to do even non-custom-shader
// optimization for plaidsea, they need to have
// the same Handle<Material> and same Handle<Mesh>
// for more info, see:
// https://bevy.org/examples/shaders/custom-shader-instancing/
// I need to assert this is actually true for the plaidsea.
// For now I've made ten variants and the framerate still
// seems solid when zoomed out.
fn init_plaidsea(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();
    let mut mesh_palette = Vec::new();
    for _ in 0..NUM_TILES {
        let mut cur_plane = Mesh::from(
            Plane3d {
                normal: Dir3::Z,
                half_size: Vec2::new(
                    GRID_SIZE as f32 / 2.,
                    GRID_SIZE as f32 / 2.,
                ),
                ..default()
            }
            .mesh(),
        );
        let vertex_colors: Vec<[f32; 4]> = vec![
            get_random_color().to_f32_array(),
            get_random_color().to_f32_array(),
            get_random_color().to_f32_array(),
            get_random_color().to_f32_array(),
        ];
        cur_plane.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);
        let exciting_mesh = meshes.add(cur_plane);
        mesh_palette.push(exciting_mesh);
    }
    let repeat_material = materials.add(StandardMaterial {
        base_color: Color::from(PURPLE),
        ..default()
    });
    // we have MAP_SIZE for both width and depth
    for y in ((-1 * physics::MAP_SIZE as i32 / 2
        + ((0.5 * GRID_SIZE as f32) as i32))
        ..=(physics::MAP_SIZE as i32 / 2))
        .step_by(GRID_SIZE as usize)
    {
        for x in ((-1 * physics::MAP_SIZE as i32 / 2
            + ((0.5 * GRID_SIZE as f32) as i32))
            ..=(physics::MAP_SIZE as i32 / 2))
            .step_by(GRID_SIZE as usize)
        {
            let random_plaid_index = rng.random_range(0..NUM_TILES) as usize;
            let center = Vec3::new(x as f32, y as f32, 0.0);
            commands.spawn((
                Mesh3d(mesh_palette[random_plaid_index].clone()),
                MeshMaterial3d(repeat_material.clone()),
                Transform::from_xyz(center.x, center.y, -1.0), //.with_scale(Vec3::splat(10. as f32)),
            ));
        }
    }
}

// FIXME(skend): use tab though
// also need a cooldown of like .5 seconds to stop multi-press
// TODO(skend): while a cooldown could be good for some cases,
// extreme bevy tutorial suggests checking for the pressed key
// not being pressed, which would then switch a bool and allow
// the key to be pressed again. so each "new press" would count
// which is the behavior we would expect. it would be more
// responsive-feeling than the cooldown idea.
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
