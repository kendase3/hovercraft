use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use serde::Deserialize;
use std::collections::HashMap;
use std::{fs, io};

const CAMERA_DEFAULT_SIZE: f32 = 100.;
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

#[derive(Resource)]
struct MinotaurAssets {
    standard_font: Handle<Font>,
}

fn get_usual_textfont(font: Handle<Font>) -> TextFont {
    TextFont {
        font: font,
        font_size: FONT_SIZE * 10.,
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
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

// the screen is ostensibly divided into lines
fn write_to_line(
    contents: String,
    line_num: u32,
    window: &Window,
    mut commands: Commands,
    minotaur_assets: Res<MinotaurAssets>,
) {
    let font = get_usual_textfont(minotaur_assets.standard_font.clone());
    commands.spawn((
        Text::new(contents),
        font,
        TextColor(Color::srgb(1., 1., 1.)),
        // NB(skend): needed to make font look prettier
        Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.1)),
        TextLayout {
            justify: Justify::Left,
            // this does not really seem to work
            //linebreak: LineBreak::WordBoundary,
            linebreak: LineBreak::AnyCharacter,
        },
        Node {
            align_self: AlignSelf::Start,
            ..default()
        },
        // ostensibly should have a red background, natch?
        //BackgroundColor(Color::srgb(1., 0., 0.,)),
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
    let text_font = get_usual_textfont(font.clone());
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
    let minotaur_assets = MinotaurAssets {
        standard_font: font,
    };
    // NB(skend): make a square behind the text, both because we will use background colors
    // and as a way to find out what font size is correct for our cell size
    // exactly fill the screen
    let background_rect = Rectangle::new(CAMERA_DEFAULT_SIZE * aspect_ratio, CAMERA_DEFAULT_SIZE);
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
        println!("room id is {}, room name is {}, room description is {}, player start is {}", room.id, room.name, room.description, room.start);
        world.rooms.insert(room.id.clone(), room.clone());
    }
    // find the player start and save it to state
    world.state.player_loc = world.get_start_id();
    // the world needs to be blitted
    commands.spawn((BlitState { is_dirty: true },));
    commands.insert_resource(minotaur_assets);
}

fn update(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
    windowq: Query<&Window>,
    mut commands: Commands,
    minotaur_assets: Res<MinotaurAssets>,
    mut blitq: Query<&mut BlitState>,
    compq: Query<(&Text, &ComputedNode), Changed<ComputedNode>>,
) {
    for (text, cn) in &compq {
        println!("text {} is {} pixels wide", text.0, cn.size.x);
    }

    let mut blitstate = blitq.single_mut().unwrap();
    if blitstate.is_dirty {
        let w = windowq.single().unwrap();
        // for simplicity, let's just pretend w == h for starters
        // what if we have a very long string
        let aspect_ratio = w.width() / w.height();
        // the screen is...10 characters tall? how many characters wide?
        // i really just need to write content in update not setup.
        // it is silly to write a lot of logic in setup about writing
        write_to_line(
            "darkness was cheap and scrooge liked it".to_string(),
            0,
            &w,
            commands,
            minotaur_assets,
        );
        blitstate.is_dirty = false;
    }
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if keys.just_pressed(KeyCode::KeyW) {
        println!("w pressed!");
    }
}
