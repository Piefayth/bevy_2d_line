#import bevy_sprite::mesh2d_view_bind_group
#import bevy_sprite::mesh2d_struct
#import bevy_sprite::mesh2d_functions::{get_world_from_local, mesh2d_position_local_to_world, mesh2d_position_world_to_clip}

struct LineVertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec2<f32>,
    @location(2) miter: f32,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> line_material: LineMaterial;

struct LineMaterial {
    thickness: f32,
};

@vertex
fn vertex(vertex: LineVertex, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    let model = get_world_from_local(instance_index);
    let world_position = mesh2d_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    
    let thickness = line_material.thickness;
    let expanded_position = world_position.xy + vertex.normal * thickness * vertex.miter;
    
    out.clip_position = mesh2d_position_world_to_clip(vec4<f32>(expanded_position, world_position.zw));
    out.color = vertex.color;
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
