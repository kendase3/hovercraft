use crate::physics;

use bevy::prelude::*;

pub fn get_laser_vertices(
    laser_origin: Vec2,
    laser_dest: Vec2,
) -> Option<(Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>)> {
    let coordpair = physics::CoordPair {
        center: laser_origin,
        exterior: laser_dest,
    };
    None
}
