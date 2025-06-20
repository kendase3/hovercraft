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

struct TargetMaterial {
    border_width: f32,
};

@group(0) @binding(0)
var<uniform> material: TargetMaterial;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

// vertex shader from example shader
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let world_from_local = get_world_from_local(vertex.instance_index);
    out.clip_position = mesh2d_position_local_to_clip(world_from_local, vec4<f32>(vertex.position, 1.0));
    return out;
}

//fragment shader
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    // calc distances from edge of square, range 0 to 1
    let dist_x_left = uv.x;
    let dist_x_right = 1.0 - uv.x;
    let dist_y_bottom = uv.y;
    let dist_y_top = 1.0 - uv.y;
    // closest to outside edge for this spot
    let min_dist = min(min(dist_x_left, dist_x_right), min(dist_y_bottom, dist_y_top));
    if min_dist < material.border_width {
        // return red opaque border
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    } else {
        // return gooey transparent center
        return vec4<f32>(1.0, 1.0, 1.0, 0.0);
    }
}
