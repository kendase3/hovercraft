
#import bevy_pbr::mesh_vertex_output
#import bevy_render::view::ViewUniform
#import bevy_render::globals::GlobalUniform
#import bevy_pbr::mesh_functions
#import bevy_pbr::pbr_lighting

struct LaserMaterial {
    amplitude: f32,
    frequency: f32,
    base_color: vec4<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct FragmentInput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>, // Pass world normal for lighting
    @location(2) uv: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> material: LaserMaterial;
@group(1) @binding(0)
var<uniform> view: ViewUniform;
@group(2) @binding(0)
var<uniform> global: GlobalUniform;

@vertex
fn vertex_shader(
    in: VertexInput,
    @builtin(instance_index) instance_index: u32,
) -> FragmentInput {
    var out: FragmentInput;
    // then do stuff
    out.uv = in.uv;
    out.clip_position = view.view_proj * in.position;
    return out;
}
