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

use bevy::prelude::*;
use bevy::time::Fixed;
use std::f32::consts::PI;

pub const MAP_SIZE: u32 = 1_000;
pub const PLAYER_ACCEL_RATE: f32 = 1_000.;
pub const BOT_ACCEL_RATE: f32 = 4_000.;
pub const PLAYER_MAX_VELOCITY: f32 = 40.;
pub const BOT_MAX_VELOCITY: f32 = 20.;

// the fewer of these, the farther away along the perimeter we aim
// the less perfect circle it will be but the less often we have
// to compute
const NODES_PER_ORBIT: f32 = 40.;
// how far ahead we look in terms of angle distance
const RADIANS_AHEAD: f32 = 2. * PI / NODES_PER_ORBIT;

#[derive(Copy, Clone, Default)]
pub struct Polar {
    pub r: f32,
    pub theta: f32,
}

impl From<CoordPair> for Polar {
    fn from(coords: CoordPair) -> Self {
        let delta_vector = coords.exterior - coords.center;
        let radians = delta_vector.y.atan2(delta_vector.x);
        let distance = delta_vector.length();

        Polar {
            r: distance,
            theta: radians,
        }
    }
}

pub struct CoordPair {
    pub center: Vec2,
    pub exterior: Vec2,
}

impl From<Polar> for Vec2 {
    fn from(polar: Polar) -> Self {
        // note that this is polar from the reference point, not the upper-left corner
        Vec2 {
            x: polar.r * polar.theta.cos(),
            y: polar.r * polar.theta.sin(),
        }
    }
}

// the stock polar will assume its reference point is 0
// this function adds the offset of whatever the other point has
pub fn polar_to_cartesean_plus_point(polar: Polar, center: Vec2) -> Vec2 {
    let relative_cartesean = Vec2::from(polar);
    relative_cartesean + center
}

pub fn orbit(
    cur_location: Vec2,
    target_location: Vec2,
    orbit_distance: f32,
) -> Vec2 {
    // first we'll find our polar coordinate
    let polar = Polar::from(CoordPair {
        exterior: cur_location,
        center: target_location,
    });
    // then we'll find our destination polar coordinate
    // we add our desired angle amount and use the destination radius
    let dest_polar = Polar {
        r: orbit_distance,
        theta: (polar.theta + RADIANS_AHEAD) % (2. * PI),
    };
    // then we'll convert that to cartesean
    polar_to_cartesean_plus_point(dest_polar, target_location)
}

// notably for now we're only using X and Y
// second arg is max velocity
#[derive(Component, Debug, Default)]
pub struct Velocity(pub Vec3, pub f32);

// second arg is acceleration rate
#[derive(Component, Debug, Default)]
pub struct Acceleration(pub Vec3, pub f32);

// FIXME(skend): acceleration is NaN the very first time
// with this func commented out, everything works as expected
pub fn apply_acceleration(
    mut query: Query<(&mut Velocity, &Acceleration)>,
    fixed_time: Res<Time<Fixed>>,
) {
    let dt = fixed_time.delta_secs();

    for (mut vel, accel) in &mut query {
        vel.0 += accel.0 * dt;
    }
}

// weirdly this one also sets velocity to zero if at edge
pub fn apply_velocity(
    mut query: Query<(&mut Transform, &mut Velocity)>,
    fixed_time: Res<Time<Fixed>>,
) {
    let dt = fixed_time.delta_secs();
    for (mut transform, mut vel) in &mut query {
        let mut actual_vel = vel.0;
        // vel.1 = max vel
        if vel.0.length() > vel.1 {
            actual_vel = vel.0.normalize();
            actual_vel = actual_vel * vel.1;
        }
        transform.translation += actual_vel * dt;
        let limit = Vec3::splat(MAP_SIZE as f32 / 2.);
        transform.translation = transform.translation.clamp(-limit, limit);
        vel.0 = update_vel_if_edge(transform.translation, vel.0);
    }
}

fn update_vel_if_edge(cur_location: Vec3, cur_vel: Vec3) -> Vec3 {
    let mut ret = cur_vel;
    let abs_location = cur_location.abs();
    let limit = Vec3::splat(MAP_SIZE as f32 / 2.);
    if abs_location.x == limit.x {
        ret.x = 0.;
    }
    if abs_location.y == limit.y {
        ret.y = 0.;
    }
    ret
}
