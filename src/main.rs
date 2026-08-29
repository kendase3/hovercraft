// Copyright 2026 Google LLC
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

use bevy::camera::ScalingMode;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::FontSmoothing;
use bevy::text::TextLayoutInfo;
use bevy::window::WindowMode;
use serde::Deserialize;
use std::collections::HashMap;
use std::{fs, io};

// the screen is defined as being 100 fauxpixels tall and aspect_ratio times this wide
const CAMERA_DEFAULT_SIZE: f32 = 100.;

// TODO(skend): my dream is that all the font size, character height and width
// will be downstream values of this value. i want to say the number of lines of text
// that fit from top to bottom here and it makes it for me.
const TARGET_LINES: u32 = 10;
const STARTING_FONT_SIZE: u32 = 10;

// FIXME(skend): likely wrong. this is from the pre-text2d era when i was using UI coordinates not
// screen coordinates
const FAUXPIXELS_PER_CHAR_WIDTH: f32 = 6.25;
// height of the largest letter
const FONT_SIZE: f32 = 10.; // what this means is the font will be 10 percent of the screen currently

#[derive(Debug, Deserialize)]
struct RoomList {
    #[serde(rename = "room")]
    rooms: Vec<Room>,
}

#[derive(Clone, Debug, Deserialize)]
struct Room {
    id: String,
    name: String,
    description: String,
    #[serde(default)]
    start: bool,
    #[serde(default)]
    exit: bool,
}

#[derive(Clone, Debug, Default)]
struct State {
    player_loc: String,
}

#[derive(Debug, Default)]
struct World {
    state: State,
    rooms: HashMap<String, Room>,
}

#[derive(Component, Default)]
struct BlitState {
    is_dirty: bool,
}

#[derive(Component, Default)]
struct Calibration {
    gold_font: Option<u32>,
    last_font_size: Option<u32>,
    last_height: Option<f32>,
}

#[derive(Component, Default)]
struct ScreenState {
    aspect_ratio: f32,
}

#[derive(Resource)]
struct MinotaurAssets {
    standard_font: Handle<Font>,
}

fn get_usual_textfont(font: Handle<Font>, font_size: u32) -> TextFont {
    TextFont {
        font: font,
        font_size: font_size as f32 * 10.,
        font_smoothing: FontSmoothing::AntiAliased,
        ..default()
    }
}

impl World {
    fn get_start_id(&self) -> String {
        // we'll iterate over the rooms until we find the player start
        // then return its id
        for room in self.rooms.values() {
            if room.start == true {
                return room.id.clone();
            }
        }
        println!("Error: No valid player start found!");
        return String::new();
    }
    fn print_cur_prompt(&self) {
        let cur_room = &self.rooms[&self.state.player_loc];
        println!("{}\n\n{}", cur_room.name, cur_room.description);
    }
    fn game_over(&self) -> bool {
        // could be any number of criteria
        // for now we'll just check if the player is at the exit
        self.rooms[&self.state.player_loc].exit
    }
}

#[derive(PartialEq)]
enum Instruction {
    Quit,
    North,
    South,
    East,
    West,
    Invalid,
}

fn get_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn get_instruction() -> Instruction {
    let input = get_input();
    match input.as_str() {
        "q" | "Q" | "quit" | "Quit" | "exit" | "Exit" => Instruction::Quit,
        "n" | "N" | "north" | "North" => Instruction::North,
        "s" | "S" | "south" | "South" => Instruction::South,
        "e" | "E" | "east" | "East" => Instruction::East,
        "w" | "W" | "west" | "West" => Instruction::West,
        &_ => Instruction::Invalid,
    }
}

fn main() {
    App::new()
        //.add_plugins(DefaultPlugins)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "mino2r".into(),
                mode: WindowMode::BorderlessFullscreen(
                    MonitorSelection::Primary,
                ),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        //.insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
        .run();
}

fn chunkify_strings(input: String, aspect_ratio: f32) -> Vec<String> {
    // first we need to find the number of characters each line can be
    // each character is 8.14 faux-pixels
    let camera_width = CAMERA_DEFAULT_SIZE * aspect_ratio;
    let char_width = (camera_width / FAUXPIXELS_PER_CHAR_WIDTH) as usize;

    let chars: Vec<char> = input.chars().collect();
    // FIXME(skend): pass in screen length in chars or
    // compute it based on aspect ratio here
    chars
        .chunks(char_width)
        //.chunks(100)
        //.chunks(1)
        .map(|line| line.iter().collect::<String>())
        .collect()
}

fn buffer_to_monostring(buffer: Vec<String>) -> String {
    let mut output = String::from("");
    for line in buffer {
        output.push_str(&line);
        output.push_str("\n");
    }
    output
}

// the screen is ostensibly divided into lines
fn write_to_line(
    contents: String,
    aspect_ratio: f32,
    window: &Window,
    mut commands: Commands,
    minotaur_assets: Res<MinotaurAssets>,
    font_size: u32,
) {
    let vert_offset = 50.;
    let horiz_offset = -1. * vert_offset * aspect_ratio;
    let font =
        get_usual_textfont(minotaur_assets.standard_font.clone(), font_size);
    // FIXME(skend): i am only supposed to run this once in setup
    // then query for it later
    commands.spawn((
        Text2d::new(contents),
        font,
        Anchor::TOP_LEFT,
        TextColor(Color::srgb(1., 1., 1.)),
        // NB(skend): scaled up then down to make font look prettier
        Transform::from_xyz(horiz_offset, vert_offset, 0.)
            .with_scale(Vec3::splat(0.1)),
        NoFrustumCulling,
        TextLayout {
            justify: Justify::Left,
            linebreak: LineBreak::NoWrap,
        },
    ));
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windowq: Query<&Window>,
) {
    // font setup part
    let font = asset_server.load("fonts/DejaVuSansMono.ttf");
    // FIXME(skend): the actual font i want is the text_font not the font
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
    let w = windowq.single().unwrap();
    let aspect_ratio = w.width() / w.height();
    commands.spawn((ScreenState { aspect_ratio }));
    let minotaur_assets = MinotaurAssets {
        standard_font: font,
    };
    // NB(skend): make a square behind the text, both because we will use background colors
    // and as a way to find out what font size is correct for our cell size
    // exactly fill the screen
    let background_rect = Rectangle::new(
        CAMERA_DEFAULT_SIZE * aspect_ratio,
        CAMERA_DEFAULT_SIZE,
    );
    let rectmesh = meshes.add(background_rect);
    commands.spawn((
        Mesh2d(rectmesh),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 0.))),
    ));

    // world file import part
    let fil = "world.toml";
    let contents = fs::read_to_string(fil).unwrap();
    let mut roomlist: RoomList = toml::from_str(&contents).unwrap();
    let mut world = World::default();
    for room in roomlist.rooms.iter_mut() {
        println!(
            "room id is {}, room name is {}, room description is {}, player start is {}",
            room.id, room.name, room.description, room.start
        );
        world.rooms.insert(room.id.clone(), room.clone());
    }
    // find the player start and save it to state
    world.state.player_loc = world.get_start_id();
    // the world needs to be blitted
    commands.spawn((BlitState { is_dirty: true },));
    commands.spawn((Calibration::default()));
    commands.insert_resource(minotaur_assets);

    // FIXME(skend): so then i no longer need get_usual_textfont?
    // this is a rather large reorg.
    let vert_offset = 50.;
    let horiz_offset = -1. * vert_offset * aspect_ratio;
    let font =
        get_usual_textfont(minotaur_assets.standard_font.clone(), font_size);
    // we will make the main text block that fills the screen. for starters it will be blank
    commands.spawn((
        Text2d::new(contents),
        font,
        Anchor::TOP_LEFT,
        TextColor(Color::srgb(1., 1., 1.)),
        // NB(skend): scaled up then down to make font look prettier
        Transform::from_xyz(horiz_offset, vert_offset, 0.)
            .with_scale(Vec3::splat(0.1)),
        NoFrustumCulling,
        TextLayout {
            justify: Justify::Left,
            linebreak: LineBreak::NoWrap,
        },
    ));
}

fn update(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
    windowq: Query<&Window>,
    mut commands: Commands,
    minotaur_assets: Res<MinotaurAssets>,
    mut blitq: Query<&mut BlitState>,
    mut screenq: Query<&mut ScreenState>,
    compq: Query<(&Text, &ComputedNode), Changed<ComputedNode>>,
    mut calibq: Query<&mut Calibration>,
    mut alreadyregretmakingthisq: Query<(&TextLayoutInfo, &Transform)>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if keys.just_pressed(KeyCode::KeyW) {
        println!("w pressed!");
    }
    // begin text calibration section
    // FIXME(skend): i believe we need a force blit or something
    // my logic below is getting the same value for text size even
    // after growing or shrinking
    let mut calibration = calibq.single_mut().unwrap();
    if calibration.gold_font.is_none() {
        // confusingly, i think we need to start by looking
        // at the results from the previous iteration
        // before we draw anything this time
        if calibration.last_font_size.is_none() {
            calibration.last_font_size = Some(STARTING_FONT_SIZE);
            return;
        }
        let w = windowq.single().unwrap();
        let window_width = w.width();
        let window_height = w.height();
        for (layout, transform) in &alreadyregretmakingthisq {
            let width = layout.size.x;
            let height = layout.size.y;
            println!("width = {}, height = {}", width, height);
            println!(
                "window width = {}, window height = {}",
                window_width, window_height
            );
            if width == 0. {
                println!("this was zero probably more often than you wanted!");
                continue;
            }
            // this works so now i just need to be able to find the screen
            // width and height

            // we want to say something like: we continue if window height is larger
            // and it's larger within one 'line' of text s.t. we could not fit another
            // so if we have 10 lines, it's within 10 percent.
            let wiggle = 1.0 / TARGET_LINES as f32 * 1.5; // so if ten lines, .1. then we give it 1.5x because we live in an imperfect world
            // testing if we are too big is easy
            if window_height > height {
                // if our screen is 1000 pixels tall and our font is 900 pixels tall
                let ratio_under = 1. - height as f32 / window_height;
                if ratio_under <= wiggle {
                    // we passed
                    calibration.gold_font =
                        Some(calibration.last_font_size.unwrap());
                } else {
                    println!("growing font");
                    calibration.last_font_size =
                        Some(calibration.last_font_size.unwrap() + 1);
                }
            } else {
                println!("shrinking font");
                calibration.last_font_size =
                    Some(calibration.last_font_size.unwrap() - 1);
            }
        }

        // we are going to have a text calibration component.
        // we will render the calibration string and we want it
        // to fit within error margins perfectly to our vertical screen
        let mut height_test_str = String::from("");
        for i in 0..TARGET_LINES {
            height_test_str.push_str("a\n");
        }
        let w = windowq.single().unwrap();
        // for simplicity, let's just pretend w == h for starters
        // what if we have a very long string
        let aspect_ratio = w.width() / w.height();
        // the screen is...10 characters tall? how many characters wide?
        // i really just need to write content in update not setup.
        // it is silly to write a lot of logic in setup about writing
        write_to_line(
            height_test_str,
            aspect_ratio,
            &w,
            commands,
            minotaur_assets,
            calibration.last_font_size.unwrap(),
        );
        return;
    }
    // end text calibration section

    for (text, cn) in &compq {
        println!("text {} is {} pixels wide", text.0, cn.size.x);
    }
    let mut screenstate = screenq.single_mut().unwrap();
    //println!(
    //    "the screen is {} faux-pixels tall and {} faux-pixels wide",
    //    CAMERA_DEFAULT_SIZE,
    //    CAMERA_DEFAULT_SIZE * screenstate.aspect_ratio
    //);
    // even though we've shown the text will autowrap
    // we should manually split up the lines ourselves.
    // how do we know how many of our faux-pixels wide our text is?
    // guess and check?
    let description = "darkness was cheap and scrooge liked it";
    // we fit 21 characters there (and change)
    // the screen is 171 faux-pixels wide
    // each character is 8.14 faux-pixels
    // so now we want a utility function that will turn one string into an array of strings that
    // will fit within the current screen width. then we can append them to the circular buffer.

    let mut blitstate = blitq.single_mut().unwrap();
    if blitstate.is_dirty {
        let w = windowq.single().unwrap();
        // for simplicity, let's just pretend w == h for starters
        // what if we have a very long string
        let aspect_ratio = w.width() / w.height();
        // the screen is...10 characters tall? how many characters wide?
        // i really just need to write content in update not setup.
        // it is silly to write a lot of logic in setup about writing
        let lines_vec =
            chunkify_strings(description.to_string(), aspect_ratio);
        let megastring = buffer_to_monostring(lines_vec);
        write_to_line(
            megastring,
            aspect_ratio,
            &w,
            commands,
            minotaur_assets,
            calibration.gold_font.unwrap(),
        );
        blitstate.is_dirty = false;
    }
}
