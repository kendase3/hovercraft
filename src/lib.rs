use bevy::prelude::*;
use std::f32::consts::PI;

// the fewer of these, the farther away along the perimeter we aim
// the less perfect circle it will be but the less often we have
// toc compute
const NODES_PER_ORBIT: f32 = 40.;
// how far ahead we look in terms of angle distance
const RADIANS_AHEAD: f32 = 2. * PI / NODES_PER_ORBIT;

// TODO(skend): determine contract for this: ultimately we want to get a destination given our
// location, our orbit target's location, the orbit distance, and clockwise/counterclockwise
//

struct Polar {
    r: f32,
    theta: f32,
}

// TODO(skend): make these from and into for the above polar struct
fn cartesean_to_polar(exterior: Vec2, center: Vec2) -> Polar {
    let delta_vector = exterior - center;
    let radians = delta_vector.y.atan2(delta_vector.x);
    let distance = delta_vector.length();

    Polar {
        r: distance,
        theta: radians,
    }
}

fn polar_to_cartesean(polar: &Polar) -> Vec2 {
    // note that this is polar from the reference point, not the upper-left corner
    Vec2 {
        x: polar.r * polar.theta.cos(),
        y: polar.r * polar.theta.sin(),
    }
}

fn polar_to_cartesean_plus_point(polar: Polar, center: Vec2) -> Vec2 {
    let relative_cartesean = polar_to_cartesean(&polar);
    relative_cartesean + center
}

// the first thing to learn is how this function is supposed to access the information
// i believe it's being called out to, so it would be passed the information it needs
pub fn orbit(
    cur_location: Vec2,
    target_location: Vec2,
    orbit_distance: f32,
) -> Vec2 {
    // first we'll find our polar coordinate
    let polar = cartesean_to_polar(cur_location, target_location);
    // then we'll find our destination polar coordinate
    // we add our desired angle amount and use the destination radius
    let dest_polar = Polar { r: orbit_distance, theta: (polar.theta + RADIANS_AHEAD) % (2. * PI)};
    // then we'll convert that to cartesean
    polar_to_cartesean_plus_point(dest_polar, target_location)
}
