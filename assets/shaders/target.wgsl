#import bevy_pbr::mesh_view_bindings
#import bevy_render::view_transform::VERT_POS_TO_CLIP

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct TargetMaterial {
    color_opaque: vec4<f32>,
    color_transparent: vec4<f32>,
    border_width: f32,
};

// bevy's material uniforms?
@group(2) @binding(0)
var<uniform> material: TargetMaterial;

@group(0) @binding(0)
var<uniform> view: bevy_pbr::ViewUniform;

@group(2) @binding(0)
var<uniform> mesh: bevy_pbr::MeshUniform;

// vertex shader
@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = mesh.transform * vec4(in.position, 1.0);
    out.clip_position = view.transform * world_position; //passthrough
    out.uv = in.uv; //passthrough
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
        return material.color_opaque;
    } else {
        return material.color_transparent;
    }
}
