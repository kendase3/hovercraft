#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_render::view_transform::VERT_POS_TO_CLIP
#import bevy_pbr::pbr_bindings
#import bevy_pbr::pbr_functions
#import bevy_pbr::pbr_types
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::utils
#import bevy_pbr::fog
#import bevy_pbr::shadows
#import bevy_pbr::pbr_ambient
#import bevy_pbr::clustered_forward
// from https://github.com/bevyengine/bevy/discussions/8498

#import bevy_sprite::{
    mesh2d_view_bindings::globals,
    mesh2d_functions::{get_world_from_local, mesh2d_position_local_to_clip},
}

//fragment shader
@fragment
fn fragment() -> @location(0) vec4<f32> {
    // half transparent red
    return vec4<f32>(1.0, 0.0, 0.0, 0.5);
}
