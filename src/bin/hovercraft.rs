// Copyright 2023 Google LLC
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

// This is a giant file that does everything.

use anyhow::{Error, Result};
use chrono::{DateTime, Duration, Utc};
use hovercraft::Pair;
use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{TextureQuery, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::Sdl;
use std::f64::consts::PI;
use std::fmt;
use std::path::Path;

static WIGGLE_PIXELS: u32 = 400;
pub const SCREEN_ROWS: u32 = 24;
pub const SCREEN_COLUMNS: u32 = 80;

// from rust sdl ttf example
// properly coerces the right mix of i32 and u32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

// the current existing user navigation command
// the ship should execute every think
// FIXME(ken): use approach and keepdistance types
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum ShipCommand {
    Approach(usize),
    Orbit(usize, f64),
    KeepDistance(usize, f64),
}

#[derive(Debug, Clone, Copy)]
struct Snack {
    location: Pair<f64>,
}

fn command_to_index(command: Option<ShipCommand>) -> Option<usize> {
    // TODO(ken): use distance values in these
    match command {
        None => None,
        Some(ShipCommand::Approach(index)) => Some(index),
        Some(ShipCommand::Orbit(index, _distance)) => Some(index),
        Some(ShipCommand::KeepDistance(index, _distance)) => Some(index),
    }
}

/// reduce an angle to no more than 2PI radians
// TODO(ken): maybe this should be +/- 1PI rads
fn simplify(angle: f64) -> f64 {
    angle % (2.0 * PI)
}

fn font_fits(
    font_size: u16,
    canvas: &WindowCanvas,
    font_path: &Path,
    allowed_width: u32,
    allowed_height: u32,
    ttf_context: &Sdl2TtfContext,
) -> Result<bool, String> {
    let texture_creator = canvas.texture_creator();
    let font = ttf_context.load_font(font_path, font_size)?;
    let surface = font
        .render("J") // j is a pretty big letter imo
        .blended(Color::RGB(255, 255, 255))
        .map_err(|e| e.to_string())?;
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
    // oh neat, we can find out how big the text is
    let TextureQuery { width, height, .. } = texture.query();

    // then we need to learn the width/height of the screen
    // then we need to learn how many divisions we plan on slicing that into
    if allowed_width >= width && allowed_height >= height {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn get_fontsize(
    canvas: &WindowCanvas,
    font_path: &Path,
    allowed_width: u32,
    allowed_height: u32,
    ttf_context: &Sdl2TtfContext,
) -> Result<u16, String> {
    let mut highest_fit = None;
    for fontsize in 8..128 {
        if font_fits(
            fontsize,
            canvas,
            font_path,
            allowed_width,
            allowed_height,
            ttf_context,
        )? {
            // it fit
            highest_fit = Some(fontsize);
        } else {
            // did not fit
            break;
        }
    }
    let fontsize = if let Some(x) = highest_fit {
        x
    } else {
        println!("no valid fontsize found!");
        8
    };
    println!("fontsize = {}", fontsize);
    Ok(fontsize)
}

fn screenify(input: Pair<f64>, pixels: Pair<i64>) -> Pair<i16> {
    // well, if you want to move the offset of a polar coordinate, that's
    // as simple as first changing its center/0 to whatever coordinate
    // value you want

    // similarly, if you want to rescale a polar universally, just weight its
    // r value with multiplication by some constant

    // if you want to rescale differently for x and y, reweight it
    // once it's broken into cartesian

    // input in the form of a value between -1 and +1
    // pixels: length or width in pixels
    // output in the form of a value between 0 and length/width

    // first adjust -1 -> +1 to 0 -> 1
    // assumes radius max value is 1
    let x = (input.x + 1.0) / 2.0;
    let y = (input.y + 1.0) / 2.0;
    // then multiply by the respective dimensions

    // i feel like i should just use height as the unit
    // for both
    let x_int = x * pixels.y as f64;
    let y_int = y * pixels.y as f64;

    Pair::new(x_int as i16, y_int as i16)
}

pub trait Targetable: Blittable {
    fn is_targeted(&self) -> bool;
    fn set_targeted(&mut self, input: bool);
    fn blit_target(
        &self,
        canvas: &mut WindowCanvas,
        sdl: &Sdl,
        camera: &Camera,
    ) -> Result<()> {
        let square_vertices = vec![
            Polar::new(1.0, 0.25 * PI),
            Polar::new(1.0, 0.75 * PI),
            Polar::new(1.0, 1.25 * PI),
            Polar::new(1.0, 1.75 * PI),
        ];
        let details = BlitDetails {
            filled: false,
            rotate: false,
            color: Some(Color::RGB(255, 0, 0)),
        };
        self.blit_vertices(&square_vertices, canvas, sdl, camera, details)?;
        Ok(())
    }
    fn blit(
        &self,
        canvas: &mut WindowCanvas,
        sdl: &Sdl,
        camera: &Camera,
    ) -> Result<()> {
        Blittable::blit(self, canvas, sdl, camera)?;
        if self.is_targeted() {
            self.blit_target(canvas, sdl, camera)?;
        }

        Ok(())
    }
}

pub struct BlitDetails {
    filled: bool,
    rotate: bool,
    color: Option<Color>,
}

pub trait Blittable {
    fn find_angle(&self, start: Pair<f64>, dest: Pair<f64>) -> f64 {
        // we now want to know:
        // what's the angle from the perspective of the ship?
        // we'll use atan2 again
        f64::atan2(dest.y - start.y, dest.x - start.x)
    }
    fn get_angle(&self) -> f64;
    fn get_location(&self) -> Pair<f64>;
    fn get_size(&self) -> f64;
    fn get_vertices(&self) -> &Vec<Polar>;
    fn get_color(&self) -> Option<Color>;
    fn blit_vertices(
        &self,
        vertices: &[Polar],
        canvas: &mut WindowCanvas,
        sdl: &Sdl,
        camera: &Camera,
        details: BlitDetails,
    ) -> Result<()> {
        let screen_dim = get_screen_size(sdl);
        let pixels_per_km = camera.pixels_per_km(&screen_dim);
        let radius_in_km = self.get_size();
        let radius_in_pixels = radius_in_km * pixels_per_km;
        let pixel_dim = Pair {
            x: radius_in_pixels as i64,
            y: radius_in_pixels as i64,
        };

        // we need to collect an array of polar coords
        let mut screen_x_vec = Vec::new();
        let mut screen_y_vec = Vec::new();
        for polar in vertices.iter() {
            let mut polar_mod = *polar;
            if details.rotate {
                polar_mod.angle += self.get_angle();
            }
            // our polars -> relative cartesian
            let relative_cart = polar_mod.as_cartesian();
            // then we need to convert them into screen cartesian
            let screen_cart = screenify(relative_cart, pixel_dim);
            screen_x_vec.push(screen_cart.x);
            screen_y_vec.push(screen_cart.y);
        }

        // note that location is in km, relative to the world
        let location = self.get_location();

        // and what is the max x/y? and is the ship within it?
        // TODO(ken): move this logic into camera?
        let camera_min = Pair {
            x: camera.center.x - (camera.km_per_width(&screen_dim) / 2.0),
            y: camera.center.y - (camera.km_per_height / 2.0),
        };
        let camera_max = Pair {
            x: camera.center.x + (camera.km_per_width(&screen_dim) / 2.0),
            y: camera.center.y + (camera.km_per_height / 2.0),
        };
        if location.x < camera_min.x
            || location.x > camera_max.x
            || location.y < camera_min.y
            || location.y > camera_max.y
        {
            // our object is outside the bounds of our camera
            return Ok(());
        }

        // now we need to rearrange our perspective:
        // the min pair is our new zero
        // and the max pair is max_pixels (for our screen).

        let zero_offset_ship = Pair {
            x: location.x - camera_min.x,
            y: location.y - camera_min.y,
        };
        // note that up until now these values are still all in km

        let ship_in_pixels = Pair {
            x: (zero_offset_ship.x * pixels_per_km) as i16,
            y: (zero_offset_ship.y * pixels_per_km) as i16,
        };

        // we want to use the...average radius? how do we get a centroid from
        // a collection of points?
        // maybe the average value for x and y does make sense to
        // find the 'centroid' (likely not actual centroid, haven't checked)
        // of an object
        let mut x_sum = 0;
        for x in &screen_x_vec {
            x_sum += x;
        }
        let x_avg = x_sum / screen_x_vec.len() as i16;
        let mut y_sum = 0;
        for y in &screen_y_vec {
            y_sum += y;
        }
        let y_avg = y_sum / screen_y_vec.len() as i16;
        let middle_of_thing = Pair { x: x_avg, y: y_avg };
        // TODO(ken): implement operators for pair

        let object_offset = Pair {
            x: ship_in_pixels.x - middle_of_thing.x,
            y: ship_in_pixels.y - middle_of_thing.y,
        };

        for x in &mut screen_x_vec {
            *x += object_offset.x;
        }
        for y in &mut screen_y_vec {
            *y += object_offset.y;
        }

        let final_color;
        // if we passed in a color explicitly, use that

        if let Some(col) = details.color {
            final_color = col;
        } else if let Some(col) = self.get_color() {
            final_color = col
        } else {
            final_color = Color::RGB(255, 255, 255);
        }
        // then we blit with color
        if details.filled {
            canvas
                .filled_polygon(&screen_x_vec, &screen_y_vec, final_color)
                .map_err(Error::msg)?;
        } else {
            canvas
                .polygon(&screen_x_vec, &screen_y_vec, final_color)
                .map_err(Error::msg)?;
        }
        Ok(())
    }
    // remember that i store my vertices as polars,
    // so that's where we're coming from
    fn blit(
        &self,
        canvas: &mut WindowCanvas,
        sdl: &Sdl,
        camera: &Camera,
    ) -> Result<()> {
        let details = BlitDetails {
            filled: true,
            rotate: true,
            color: None,
        };
        self.blit_vertices(self.get_vertices(), canvas, sdl, camera, details)?;

        Ok(())
    }
}

#[derive(Default)]
struct World {
    // TODO(ken): introduce idea of dynamic world size
    //size: Pair<f64>,
    camera: Camera,
    ship: Ship,
    enemy: Ship,
    asteroids: Vec<Asteroid>,
    last_think: Option<DateTime<Utc>>,
    bullet_bank: BulletBank,
    target_index: Option<usize>,
}

impl World {
    fn new(size: Pair<f64>) -> World {
        let center = Pair {
            x: size.x / 2.0,
            y: size.y / 2.0,
        };
        World {
            //size,
            camera: Camera {
                km_per_height: 0.5,
                center,
            },
            ..Default::default()
        }
    }

    fn spawn_ship(&mut self, location: Pair<f64>) {
        self.ship = Ship::new(location, Some(Color::RGB(0, 255, 255)), 0.01);
    }

    fn spawn_enemy(&mut self, location: Pair<f64>) {
        self.enemy = Ship::new(location, Some(Color::RGB(255, 0, 0)), 0.03);
    }

    fn spawn_asteroid(&mut self, location: Pair<f64>) {
        let asteroid = Asteroid::new(location);
        self.asteroids.push(asteroid);
    }

    fn orbit_target(&mut self) {
        // TODO(ken): first we get the current target's index
        // interestingly this is stored on world, not ship
        // that is kind of dumb but we'll roll with it for now
        if let Some(ti) = self.target_index {
            // we update the ship's command to be orbit the current target
            self.ship.set_command(ShipCommand::Orbit(ti, 0.2));
        }
    }

    fn blit(&self, canvas: &mut WindowCanvas, sdl: &Sdl) -> Result<()> {
        Targetable::blit(&self.ship, canvas, sdl, &self.camera)?;
        Targetable::blit(&self.enemy, canvas, sdl, &self.camera)?;
        for asteroid in self.asteroids.iter() {
            Targetable::blit(asteroid, canvas, sdl, &self.camera)?;
        }
        self.bullet_bank.blit_all(canvas, sdl, &self.camera)?;
        Ok(())
    }

    // we'll think at a rate of 10hz or so
    fn needs_think(&mut self) -> bool {
        match self.last_think {
            None => {
                self.last_think = Some(Utc::now());
                true
            }
            Some(think_innards) => {
                let now = Utc::now();
                // true if time elapsed is over the threshold
                now - think_innards > Duration::milliseconds(100)
            }
        }
    }

    fn get_seconds_elapsed(&self) -> f64 {
        let now = Utc::now();
        let time_elapsed = now - self.last_think.unwrap();
        // for now we should just have the ship use velocity instead of acceleration
        let mut time_as_float = time_elapsed.num_seconds() as f64;
        time_as_float += time_elapsed.num_milliseconds() as f64 * 1.0 / 1000.0;
        time_as_float += time_elapsed.num_minutes() as f64 * 60.0;
        time_as_float
    }

    // what will change here is that the ship for example will need some
    // information about external entities
    //
    // problems for the future:
    // i think we'll ultimately want to say something like 'for each of the
    // entities in local, think'
    //
    // even bullets have to assess if they've hit anything
    // though they don't at the moment
    fn think(&mut self) {
        // a snack implements copy trait and holds (for now just one)
        // values from a ship-like object
        let ship_thinkee = self.ship.get_thinkee();
        let snack = self.get_snack(ship_thinkee);
        // last arg is destination
        self.ship
            .think(self.get_seconds_elapsed(), snack.unwrap().location);
        self.enemy
            .think(self.get_seconds_elapsed(), self.ship.location);
        self.bullet_bank.think();
        // then update last_think to now
        self.last_think = Some(Utc::now());
    }
    fn shoot(&mut self) {
        // spawn a bullet at our location
        // the bullet is moving toward the asteroid
        let mut tar_loc = None;
        if let Some(t) = self.get_target() {
            tar_loc = Some(t.get_location());
        }
        if let Some(t) = tar_loc {
            self.bullet_bank.spawn(self.ship.location, t);
        }
    }

    fn tab_target(&mut self) {
        // what is our current target index?
        if let Some(ti) = self.target_index {
            self.get_target().unwrap().set_targeted(false);
            let mut next_target = ti + 1;
            if next_target > self.asteroids.len() {
                next_target = 0;
            }
            self.target_index = Some(next_target);
        } else {
            self.target_index = Some(0);
        }
        self.get_target().unwrap().set_targeted(true);
    }
    fn get_target(&mut self) -> Option<&mut dyn Targetable> {
        if let Some(ti) = self.target_index {
            if ti == 0 {
                // the first target should be the enemy
                return Some(&mut self.enemy);
            } else {
                return Some(&mut self.asteroids[ti - 1]);
            }
        }
        None
    }
    // for a given index, return snack struct representing its state
    fn get_snack(&self, index: Option<usize>) -> Option<Snack> {
        let index = index?;
        let loc = if index == 0 {
            self.enemy.location
        } else {
            self.asteroids[index - 1].location
        };
        Some(Snack { location: loc })
    }
}

// world could eventually have a vector of players
// each with a ship and a camera
#[derive(Debug, Default, Clone, Copy)]
pub struct Camera {
    km_per_height: f64,
    center: Pair<f64>,
}

impl Camera {
    fn pixels_per_km(&self, screen_pixels: &Pair<i64>) -> f64 {
        let height_per_km = 1.0 / self.km_per_height;
        height_per_km * screen_pixels.y as f64
    }
    fn km_per_width(&self, screen_pixels: &Pair<i64>) -> f64 {
        let ratio = screen_pixels.x as f64 / screen_pixels.y as f64;
        self.km_per_height * ratio
    }
}

#[derive(Debug, Default, Clone)]
struct Ship {
    vertices: Vec<Polar>,
    angle: f64,
    location: Pair<f64>,
    size: f64,
    destination: Option<Pair<f64>>,
    color: Option<Color>,
    velocity: f64,
    targeted: bool,
    command: Option<ShipCommand>,
    // TODO(ken): consider per-entity last-think values
}

impl Targetable for Ship {
    fn set_targeted(&mut self, input: bool) {
        self.targeted = input;
    }
    fn is_targeted(&self) -> bool {
        self.targeted
    }
}

impl Blittable for Ship {
    fn get_angle(&self) -> f64 {
        self.angle
    }
    fn get_location(&self) -> Pair<f64> {
        self.location
    }
    fn get_size(&self) -> f64 {
        self.size
    }
    fn get_vertices(&self) -> &Vec<Polar> {
        &self.vertices
    }
    fn get_color(&self) -> Option<Color> {
        self.color
    }
}

impl Ship {
    fn new(location: Pair<f64>, color: Option<Color>, velocity: f64) -> Ship {
        // let's make a deterministic asteroid first
        let polars = vec![
            Polar::new(1.0, 0.0),
            Polar::new(1.0, 0.8 * PI),
            Polar::new(1.0, 1.2 * PI),
        ];
        Ship {
            vertices: polars,
            location,
            size: 0.05,
            color,
            velocity,
            command: Some(ShipCommand::Orbit(0, 0.2)),
            ..Default::default()
        }
    }

    fn get_thinkee(&self) -> Option<usize> {
        command_to_index(self.command)
    }

    fn orbit(&mut self, distance: f64, clockwise: bool, target: Pair<f64>) {
        // how far away should we set the waypoint in angular distance?
        // at 0.5 * PI we move in squares
        let rads_away = 0.05 * PI;

        // here's where things get spicy
        // let's 180 that so it's from the perspective of the asteroid looking
        // at the ship
        let angle_facing_asteroid = self.find_angle(self.location, target);

        let angle_facing_ship = simplify(angle_facing_asteroid + PI);
        // we'll add 45 degrees if we want to orbit clockwise.
        // we subtract for counter-clockwise
        let modi = if clockwise {
            rads_away
        } else {
            -1.0 * rads_away
        };
        let perp_angle = simplify(angle_facing_ship + modi);
        // we basically start at the asteroid, then add the polar coordinate
        // the easist way would be to convert the polar to a cart coord
        // then sum them
        let dest_from_asteroid =
            Polar::new(distance, perp_angle).as_cartesian();
        let dest = Pair::new(
            target.x + dest_from_asteroid.x,
            target.y + dest_from_asteroid.y,
        );
        self.destination = Some(dest);
        // we now want to know:
        // what's the angle from the perspective of the ship?
        // we'll use atan2 again
        let angle =
            f64::atan2(dest.y - self.location.y, dest.x - self.location.x);

        self.angle = angle;
    }

    fn has_arrived(&self) -> bool {
        let destination = if let Some(dest) = self.destination {
            dest
        } else {
            // if we have nowhere to go then sure we've arrived
            return true;
        };
        let close_enough = 0.04;
        let x_close_enough = f64::abs(f64::cos(self.angle) * close_enough);
        let y_close_enough = f64::abs(f64::sin(self.angle) * close_enough);
        let maybe_close_enough_x = f64::abs(self.location.x - destination.x);
        let maybe_close_enough_y = f64::abs(self.location.y - destination.y);
        // return whether or not we have arrived
        maybe_close_enough_x < x_close_enough
            && maybe_close_enough_y < y_close_enough
    }
    fn think(&mut self, seconds_elapsed: f64, dest: Pair<f64>) {
        if self.has_arrived() {
            self.orbit(0.2, true, dest);
        }

        // TODO(ken): implement acceleration/max_vel
        let km_moved = self.velocity * seconds_elapsed;
        // now we move km_moved at our angle
        // turns out both cos()/sin() and x,y match alphabetically
        // so that's like, a mnemonic-esque thing
        let x_distance = f64::cos(self.angle) * km_moved;
        let y_distance = f64::sin(self.angle) * km_moved;
        self.location.x += x_distance;
        self.location.y += y_distance;
    }
    fn set_command(&mut self, command: ShipCommand) {
        self.command = Some(command);
    }
}

impl fmt::Display for Ship {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.vertices)
    }
}

#[derive(Debug, Default, Clone)]
struct Asteroid {
    vertices: Vec<Polar>,
    angle: f64,
    location: Pair<f64>,
    size: f64,
    targeted: bool,
}

impl fmt::Display for Asteroid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.vertices)
    }
}

impl Targetable for Asteroid {
    fn set_targeted(&mut self, input: bool) {
        self.targeted = input;
    }
    fn is_targeted(&self) -> bool {
        self.targeted
    }
}

impl Blittable for Asteroid {
    fn get_angle(&self) -> f64 {
        self.angle
    }
    fn get_location(&self) -> Pair<f64> {
        self.location
    }
    fn get_size(&self) -> f64 {
        self.size
    }
    fn get_vertices(&self) -> &Vec<Polar> {
        &self.vertices
    }
    fn get_color(&self) -> Option<Color> {
        None
    }
}

impl Asteroid {
    fn new(location: Pair<f64>) -> Asteroid {
        // let's make a deterministic asteroid first
        let mut polars = Vec::new();
        let mut angle_itr = 0.0;
        let radius_itr = 1.0;
        while angle_itr < 2.0 * PI {
            polars.push(Polar::new(radius_itr, angle_itr));
            angle_itr += PI / 8.0;
        }
        Asteroid {
            vertices: polars,
            location,
            size: 0.05,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Bullet {
    vertices: Vec<Polar>,
    angle: f64,
    location: Pair<f64>,
    size: f64,
    show: bool,
}

impl fmt::Display for Bullet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.vertices)
    }
}

impl Blittable for Bullet {
    fn get_angle(&self) -> f64 {
        self.angle
    }
    fn get_location(&self) -> Pair<f64> {
        self.location
    }
    fn get_size(&self) -> f64 {
        self.size
    }
    fn get_vertices(&self) -> &Vec<Polar> {
        &self.vertices
    }
    fn get_color(&self) -> Option<Color> {
        None
    }
}

impl Bullet {
    // TODO(ken): i don't yet have the idea of bullets expiring
    // but will likely base it on range, if they hit something, etc.
    fn new(location: Pair<f64>) -> Bullet {
        // let's make a deterministic asteroid first
        let mut polars = Vec::new();
        let mut angle_itr = 0.0;
        let radius_itr = 1.0;
        while angle_itr < 2.0 * PI {
            // bullets are squares
            polars.push(Polar::new(radius_itr, angle_itr));
            angle_itr += PI / 2.0;
        }
        Bullet {
            vertices: polars,
            location,
            size: 0.01,
            // these bullets are not live, so should not be displayed
            show: false,
            ..Default::default()
        }
    }
}

#[derive(Default)]
struct BulletBank {
    ready: bool,
    // TODO(ken): the abstraction here is the bullet bank
    // is opaque and just lets you check out and check in bullets
    // it will issue a warning if a live bullet was replaced because
    // we wrapped
    bullets: Vec<Bullet>,
    next_index: usize,
    last_think: Option<DateTime<Utc>>,
}

impl BulletBank {
    // FIXME(ken): i feel like if i wanted to i could just make my own default
    // method instead of using the vanilla default and then juggling this
    // 'make sure the struct has been initialized' thing which is error-prone
    fn init(&mut self) {
        if self.ready {
            return;
        }
        let count = 1000;
        for _ in 0..count {
            self.bullets.push(Bullet::new(Pair { x: 0.0, y: 0.0 }));
        }
        self.last_think = Some(Utc::now());
        self.ready = true;
    }
    // TODO(ken): finish
    fn spawn(&mut self, start_coords: Pair<f64>, end_coords: Pair<f64>) {
        if !self.ready {
            self.init();
        }
        let bullet = &mut self.bullets[self.next_index];
        bullet.show = true;
        bullet.location = start_coords;
        bullet.angle = bullet.find_angle(start_coords, end_coords);

        // update next_index for next call to spawn()
        self.next_index += 1;
        self.next_index %= self.bullets.len();
    }
    fn think(&mut self) {
        if !self.ready {
            // FIXME(ken): this fires
            // apparently i was right about making the default thing
            self.init();
        }
        let now = Utc::now();
        // iterate over our bullets and move them
        for bullet in self.bullets.iter_mut() {
            let time_elapsed = now - self.last_think.unwrap();
            let bullet_velocity = 0.10; // in km/second

            let mut time_as_float = time_elapsed.num_seconds() as f64;
            time_as_float +=
                time_elapsed.num_milliseconds() as f64 * 1.0 / 1000.0;
            time_as_float += time_elapsed.num_minutes() as f64 * 60.0;
            let km_moved = bullet_velocity * time_as_float;
            let x_distance = f64::cos(bullet.angle) * km_moved;
            let y_distance = f64::sin(bullet.angle) * km_moved;
            bullet.location.x += x_distance;
            bullet.location.y += y_distance;
        }
        self.last_think = Some(now);
    }
    fn blit_all(
        &self,
        canvas: &mut WindowCanvas,
        sdl: &Sdl,
        camera: &Camera,
    ) -> Result<()> {
        for bullet in self.bullets.iter() {
            bullet.blit(canvas, sdl, camera)?;
        }
        Ok(())
    }
}

// we pass in the canvas we want to display on
// we pass in the world we want to display
// we pass in the camera object with the offset, zoom ratio, and rotation
// of the camera
fn display(canvas: &mut WindowCanvas, sdl: &Sdl, world: &World) -> Result<()> {
    world.blit(canvas, sdl)?;
    Ok(())
}

fn get_screen_size(sdl_context: &sdl2::Sdl) -> Pair<i64> {
    let video_subsys = sdl_context.video().unwrap();
    let dim = video_subsys.current_display_mode(0).unwrap();
    let width = dim.w - (WIGGLE_PIXELS as i32);
    let height = dim.h - (WIGGLE_PIXELS as i32);
    Pair::new(width.into(), height.into())
}

fn run_game() -> Result<(), String> {
    let font_path = Path::new("third_party/fonts/DejaVuSansMono.ttf");
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let screenheight = video_subsys.current_display_mode(0)?.h as u32;
    let screenwidth = video_subsys.current_display_mode(0)?.w as u32;
    let winheight = screenheight - WIGGLE_PIXELS;
    let winwidth = screenwidth - WIGGLE_PIXELS;
    // to clarify, these are the dimensions in pixels of each 'cell'
    let allowed_width = winwidth / SCREEN_COLUMNS;
    let allowed_height = winheight / SCREEN_ROWS;
    let char_w = allowed_width as i32;
    // the height in pixels of the character
    let char_h = allowed_height as i32;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let window = video_subsys
        .window("hovercraft", winwidth, winheight)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas =
        window.into_canvas().build().map_err(|e| e.to_string())?;
    let font_size = get_fontsize(
        &canvas,
        font_path,
        allowed_width,
        allowed_height,
        &ttf_context,
    )?;
    let texture_creator = canvas.texture_creator();
    let font = ttf_context.load_font(font_path, font_size)?;
    let text = "bogey demo";
    let surface = font
        .render(text)
        .blended(Color::RGB(255, 255, 255))
        .map_err(|e| e.to_string())?;
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
    let text_target = rect!(
        screenwidth / 2,
        screenheight / 2,
        char_w * text.len() as i32,
        char_h
    );
    let mut world = World::new(Pair { x: 1.0, y: 1.0 });
    world.spawn_ship(Pair { x: 0.4, y: 0.4 });
    world.spawn_enemy(Pair { x: 0.65, y: 0.5 });
    world.spawn_asteroid(Pair { x: 0.5, y: 0.5 });
    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'mainloop,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => world.shoot(),
                Event::KeyDown {
                    keycode: Some(Keycode::Tab),
                    ..
                } => world.tab_target(),
                Event::KeyDown {
                    keycode: Some(Keycode::O),
                    ..
                } => world.orbit_target(),
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        display(&mut canvas, &sdl_context, &world).unwrap();
        canvas.copy(&texture, None, Some(text_target))?;
        canvas.present();
        if world.needs_think() {
            world.think();
        }
    }
    Ok(())
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Polar {
    radius: f64,
    angle: f64,
}

impl Polar {
    fn new(radius: f64, angle: f64) -> Polar {
        Polar { radius, angle }
    }
    fn as_cartesian(&self) -> Pair<f64> {
        let x = self.radius * f64::cos(self.angle);
        let y = self.radius * f64::sin(self.angle);
        Pair::new(x, y)
    }
    // TODO(ken): use
    /*
    fn from_cartesian(x: f64, y: f64) -> Polar {
        let z = f64::sqrt(x * x + y * y);
        let angle = f64::atan2(y, x);
        Polar::new(z, angle)
    }
    */
}

fn main() -> Result<(), String> {
    run_game()?;
    Ok(())
}
