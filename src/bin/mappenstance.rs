use bevy::prelude::*;
// let's do a top-down view of a 8x8 grid, the height of the viewing window

fn main() {
    // first find height of viewing window
    // then divide it into 8x8 cells
    // then create a 2d array of cells with some filled, some empty
    // then render it from above
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // fill whole browser window
                fit_canvas_to_parent: true,
                // don't listen to keyboard shortcuts like F keys, ctrl+R
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.53, 0.53, 0.53)))
        //.add_systems(Startup, (startup, spawn_player))
        //.add_systems(Update, move_player)
        .run();
}
