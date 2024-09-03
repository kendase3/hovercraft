use bevy::prelude::*;

fn main() {
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
        .run();
}
