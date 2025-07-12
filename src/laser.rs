use crate::physics;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use std::f32::consts::PI;

const LASER_WIDTH: f32 = 4.0;
const LASER_HEIGHT: f32 = 0.5;

pub fn get_new_laser() -> Mesh {
    let mut ret = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let coords: Vec<[f32; 3]> = vec![[0., 0., 0.]; 8];
    let indices: Vec<u32> = vec![0; 36];
    ret.insert_attribute(Mesh::ATTRIBUTE_POSITION, coords);
    ret.insert_indices(Indices::U32(indices));
    ret
}

pub fn get_uvs() -> Vec<[f32; 2]> {
    vec![
        // near
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        // far
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        // top
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        // bottom
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        // right
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        // left
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
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

pub fn get_laser_vertices(
    laser_origin: Vec2,
    laser_dest: Vec2,
) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 2]>) {
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
        laser_vertex_1_xy.extend(-1. * LASER_HEIGHT).into(), // near bottom left
        laser_vertex_2_xy.extend(-1. * LASER_HEIGHT).into(), // near bottom right
        laser_vertex_2_xy.extend(LASER_HEIGHT).into(),       // near top right
        laser_vertex_1_xy.extend(LASER_HEIGHT).into(),       // near top left
        laser_vertex_3_xy.extend(-1. * LASER_HEIGHT).into(), // far bottom left
        laser_vertex_4_xy.extend(-1. * LASER_HEIGHT).into(), // far bottom right
        laser_vertex_4_xy.extend(LASER_HEIGHT).into(),       // far top right
        laser_vertex_3_xy.extend(LASER_HEIGHT).into(),       // far top left
    ];

    (coords, get_indices(), get_uvs())
}
