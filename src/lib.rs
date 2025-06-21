use bevy::prelude::*;

// TODO(skend): determine contract for this: ultimately we want to get a destination given our
// location, our orbit target's location, the orbit distance, and clockwise/counterclockwise
//
// the first thing to learn is how this function is supposed to access the information
// i believe it's being called out to, so it would be passed the information it needs
pub fn orbit(
    cur_location: Vec2,
    target_location: Vec2,
    distance: f32,
) -> Vec2 {
    //println!("{}", cur_location.y);
    Vec2 { x: 0.0, y: 0.0 }
}
