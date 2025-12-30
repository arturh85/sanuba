// Vertex shader

struct CameraUniform {
    position: vec2<f32>,
    zoom: f32,
    aspect: f32,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var world_texture: texture_2d<f32>;
@group(0) @binding(1)
var world_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the world texture
    // For now, simple 1:1 mapping - camera integration comes later
    let color = textureSample(world_texture, world_sampler, in.tex_coords);
    
    // Premultiply alpha for correct blending
    return vec4<f32>(color.rgb * color.a, color.a);
}
