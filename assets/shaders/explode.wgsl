
// The time since startup data is in the globals binding which is part of the mesh_view_bindings import
#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexInput,
    forward_io::VertexOutput,
}

struct ExplodeMaterial {
    explode_center: vec3<f32>,
    explode_progress: f32,
}

@group(0) @binding(0)
var<uniform> material: ExplodeMaterial;

@vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = mesh.model * vec4<f32>(in.position, 1.0);
    let direction = normalize(world_position.xyz - material.explosion_center);
    let displacement = direction * material.explosion_progress;
    let new_position = world_position.xyz + displacement;
    out.clip_position = view.view_proj* vec4<f32>(new_position, 1.0);
    out.world_position = vec4<f32>(new_position, 1.0);
    out.uv = in.uv;
    return out;
}
