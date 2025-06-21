use bevy::prelude::*;
use std::f32::consts::PI;

// the fewer of these, the farther away along the perimeter we aim
// the less perfect circle it will be but the less often we have
// toc compute
const NODES_PER_ORBIT: u32 = 10;

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

fn polar_to_cartesean(polar: Polar) -> Vec2 {
    // note that this is polar from the reference point, not the upper-left corner
    Vec2 {
        x: polar.r * polar.theta.cos(),
        y: polar.r * polar.theta.sin(),
    }
}

// the first thing to learn is how this function is supposed to access the information
// i believe it's being called out to, so it would be passed the information it needs
pub fn orbit(
    cur_location: Vec2,
    target_location: Vec2,
    orbit_distance: f32,
) -> Vec2 {
    // first we'll find our polar coordinate
    // then we'll find our destination polar coordinate
    // then we'll convert that to cartesean
    // then we'll return it
    Vec2 { x: 0.0, y: 0.0 }
}
