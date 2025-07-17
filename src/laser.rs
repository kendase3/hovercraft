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

use crate::physics;

use bevy::prelude::*;
use std::cmp::min;
use std::f32::consts::PI;

const LASER_WIDTH: f32 = 0.5;
const LASER_HEIGHT: f32 = 0.5;
const LASER_RANGE: f32 = 200.;

pub fn get_uvs() -> Vec<[f32; 2]> {
    // what if we only care about how top and bottom look
    vec![
        [1.0, 1.0], // top-right of texture
        [1.0, 0.0], // bottom-right of texture
        [0.0, 0.0], // bottom-left of texture
        [0.0, 1.0], // top-left of texture
        [0.0, 1.0], // top-left of texture
        [0.0, 0.0], // bottom-left of texture
        [1.0, 0.0], // bottom-right of texture
        [0.0, 0.0], // bottom-left of texture
    ]
}

pub fn get_indices() -> Vec<u32> {
    vec![
        0, 1, 2, 0, 2, 3, // near face triangles
        4, 7, 6, 4, 6, 5, // far face triangles
        3, 2, 6, 3, 6, 7, // top face triangles
        0, 4, 5, 0, 5, 1, // bottom face triangles
        1, 5, 6, 1, 5, 2, // right face triangles
        4, 0, 3, 4, 3, 7, // left face triangles
    ]
}

pub fn bound_on_range(polar_in: physics::Polar) -> physics::Polar {
    // TODO(skend): this is when we discover whether it is a hit or not
    // though it would not be too wasteful to just compare the two values
    // separately at another time
    let new_radius = f32::min(polar_in.r, LASER_RANGE);
    physics::Polar {
        theta: polar_in.theta,
        r: new_radius,
    }
}

pub fn get_laser_vertices(
    laser_origin: Vec2,
    mut laser_dest: Vec2,
) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 2]>) {
    let coordpair = physics::CoordPair {
        center: laser_origin,
        exterior: laser_dest,
    };
    let mut polar = physics::Polar::from(coordpair);
    // handle idea that the laser can only fire so far
    if polar.r > LASER_RANGE {
        polar = bound_on_range(polar);
        laser_dest =
            physics::polar_to_cartesean_plus_point(polar, laser_origin);
    }
    let maltheta = polar.theta + 0.5 * PI % (2. * PI);
    // oddly PEMDAS really went my way on this one, very few () required
    // maybe i don't even need those last ones but i'm too lazy
    // to look up where % falls into the order of operations and i don't
    // have it memorized
    let maltheta2 = polar.theta - 0.5 * PI + 2. * PI % (2. * PI);
    let laser_vertex_1_polar = physics::Polar {
        theta: maltheta,
        r: LASER_WIDTH / 2.,
    };
    let laser_vertex_2_polar = physics::Polar {
        theta: maltheta2,
        r: LASER_WIDTH / 2.,
    };
    // and then we'll crap out its vec2
    let laser_vertex_1_xy = physics::polar_to_cartesean_plus_point(
        laser_vertex_1_polar,
        laser_origin,
    );
    let laser_vertex_2_xy = physics::polar_to_cartesean_plus_point(
        laser_vertex_2_polar,
        laser_origin,
    );
    // and then, if we take those same polar offsets from the destination,
    // we get the other side of our rectangle
    let laser_vertex_3_xy = physics::polar_to_cartesean_plus_point(
        laser_vertex_1_polar,
        laser_dest,
    );
    let laser_vertex_4_xy = physics::polar_to_cartesean_plus_point(
        laser_vertex_2_polar,
        laser_dest,
    );

    let coords: Vec<[f32; 3]> = vec![
        laser_vertex_1_xy.extend(-1. * LASER_HEIGHT).into(), // 0, near bottom left
        laser_vertex_2_xy.extend(-1. * LASER_HEIGHT).into(), // 1, near bottom right
        laser_vertex_2_xy.extend(LASER_HEIGHT).into(), // 2, near top right
        laser_vertex_1_xy.extend(LASER_HEIGHT).into(), // 3, near top left
        laser_vertex_3_xy.extend(-1. * LASER_HEIGHT).into(), // 4, far bottom left
        laser_vertex_4_xy.extend(-1. * LASER_HEIGHT).into(), // 5, far bottom right
        laser_vertex_4_xy.extend(LASER_HEIGHT).into(), // 6, far top right
        laser_vertex_3_xy.extend(LASER_HEIGHT).into(), // 7, far top left
    ];

    (coords, get_indices(), get_uvs())
}
