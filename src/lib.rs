use bevy::prelude::*;
use std::f32::consts::PI;

// the fewer of these, the farther away along the perimeter we aim
// the less perfect circle it will be but the less often we have
// to compute
const NODES_PER_ORBIT: f32 = 40.;
// how far ahead we look in terms of angle distance
const RADIANS_AHEAD: f32 = 2. * PI / NODES_PER_ORBIT;

pub struct Polar {
    r: f32,
    theta: f32,
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
    center: Vec2,
    exterior: Vec2,
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
fn polar_to_cartesean_plus_point(polar: Polar, center: Vec2) -> Vec2 {
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
