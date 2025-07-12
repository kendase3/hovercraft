use crate::physics;

use bevy::prelude::*;
use std::f32::consts::PI;

const LASER_WIDTH: f32 = 4.0;
const LASER_HEIGHT: f32 = 0.5;

pub fn get_laser_vertices(
    laser_origin: Vec2,
    laser_dest: Vec2,
) -> Option<(Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>)> {
    let coordpair = physics::CoordPair {
        center: laser_origin,
        exterior: laser_dest,
    };
    let polar = physics::Polar::from(coordpair);
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
        [laser_vertex_1_xy.x, laser_vertex_1_xy.y, -1. * LASER_HEIGHT], // near bottom left
        [laser_vertex_2_xy.x, laser_vertex_2_xy.y, -1. * LASER_HEIGHT], // near bottom right
        [laser_vertex_2_xy.x, laser_vertex_2_xy.y, LASER_HEIGHT], // near top right
        [laser_vertex_1_xy.x, laser_vertex_1_xy.y, LASER_HEIGHT], // near top left
        [laser_vertex_3_xy.x, laser_vertex_3_xy.y, -1. * LASER_HEIGHT], // far bottom left
        [laser_vertex_4_xy.x, laser_vertex_4_xy.y, -1. * LASER_HEIGHT], // far bottom right
        [laser_vertex_4_xy.x, laser_vertex_4_xy.y, LASER_HEIGHT], // far top right
        [laser_vertex_3_xy.x, laser_vertex_3_xy.y, LASER_HEIGHT], // far top left
    ];

    let mut vertices: Vec<[f32; 3]> = vec![[0., 0., 0.]; 24];
    // TODO(skend): make sure these are all CCW
    // well we can get started i guess. let's make a face that's on the origin side
    vertices[0] =
        (laser_vertex_1_xy.x, laser_vertex_1_xy.y, LASER_HEIGHT).into();
    vertices[1] =
        (laser_vertex_1_xy.x, laser_vertex_1_xy.y, -1. * LASER_HEIGHT).into();
    vertices[2] =
        (laser_vertex_2_xy.x, laser_vertex_2_xy.y, -1. * LASER_HEIGHT).into();
    vertices[3] =
        (laser_vertex_2_xy.x, laser_vertex_2_xy.y, LASER_HEIGHT).into();
    // and just like that, we have our face at the origin with the normal headed toward the
    // laser vector
    // now we can make the top face
    vertices[4] =
        (laser_vertex_1_xy.x, laser_vertex_1_xy.y, LASER_HEIGHT).into();
    vertices[5] =
        (laser_vertex_4_xy.x, laser_vertex_4_xy.y, LASER_HEIGHT).into();
    vertices[6] =
        (laser_vertex_3_xy.x, laser_vertex_3_xy.y, LASER_HEIGHT).into();
    vertices[7] =
        (laser_vertex_2_xy.x, laser_vertex_2_xy.y, LASER_HEIGHT).into();
    // next we can make the face at the destination
    vertices[8] =
        (laser_vertex_4_xy.x, laser_vertex_4_xy.y, LASER_HEIGHT).into();
    vertices[9] =
        (laser_vertex_4_xy.x, laser_vertex_4_xy.y, -1. * LASER_HEIGHT).into();
    vertices[10] =
        (laser_vertex_3_xy.x, laser_vertex_3_xy.y, -1. * LASER_HEIGHT).into();
    vertices[11] =
        (laser_vertex_3_xy.x, laser_vertex_3_xy.y, LASER_HEIGHT).into();
    // next we can make the face on the bottom
    vertices[12] =
        (laser_vertex_1_xy.x, laser_vertex_1_xy.y, -1. * LASER_HEIGHT).into();
    vertices[13] =
        (laser_vertex_3_xy.x, laser_vertex_3_xy.y, -1. * LASER_HEIGHT).into();
    vertices[14] =
        (laser_vertex_4_xy.x, laser_vertex_4_xy.y, -1. * LASER_HEIGHT).into();
    vertices[15] =
        (laser_vertex_2_xy.x, laser_vertex_2_xy.y, -1. * LASER_HEIGHT).into();
    // then we can do the left side
    vertices[16] =
        (laser_vertex_1_xy.x, laser_vertex_1_xy.y, LASER_HEIGHT).into();
    vertices[17] =
        (laser_vertex_3_xy.x, laser_vertex_3_xy.y, LASER_HEIGHT).into();
    vertices[18] =
        (laser_vertex_3_xy.x, laser_vertex_3_xy.y, -1. * LASER_HEIGHT).into();
    vertices[19] =
        (laser_vertex_1_xy.x, laser_vertex_1_xy.y, -1. * LASER_HEIGHT).into();
    // finally the right side
    vertices[20] =
        (laser_vertex_2_xy.x, laser_vertex_2_xy.y, LASER_HEIGHT).into();
    vertices[21] =
        (laser_vertex_2_xy.x, laser_vertex_2_xy.y, -1. * LASER_HEIGHT).into();
    vertices[22] =
        (laser_vertex_4_xy.x, laser_vertex_4_xy.y, -1. * LASER_HEIGHT).into();
    vertices[23] =
        (laser_vertex_4_xy.x, laser_vertex_4_xy.y, LASER_HEIGHT).into();
    None
}
