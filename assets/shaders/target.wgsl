
#import bevy_sprite::mesh2d_vertex_output::VertexOutput                                                                   

struct TargetMaterial {
    border_width: f32,
};

@group(0) @binding(0)
var<uniform> material: TargetMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

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
        //return vec4<f32>(1.0, 0.0, 0.0, 1.0);
        return vec4<f32>(0.0, 0.0, 1.0, 0.5);
    } else {
        // return gooey transparent center
        return vec4<f32>(1.0, 1.0, 1.0, 0.0);
    }
}
