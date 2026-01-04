// Bloom Upsample Shader
// Performs upsampling with tent filter and blending using Kawase dual filtering

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

// Uniform for filter radius
struct UpsampleUniforms {
    filter_radius: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
};

@group(0) @binding(2)
var<uniform> uniforms: UpsampleUniforms;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Full-screen triangle
    var out: VertexOutput;
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(input_texture));
    let radius = texel_size * uniforms.filter_radius;

    // 9-tap tent filter (box blur approximation)
    // Weighted sampling in a tent pattern for smooth upsampling
    var result = vec4<f32>(0.0);

    // Center sample (highest weight)
    result += textureSample(input_texture, input_sampler, in.uv) * 4.0;

    // Cardinal directions (medium weight)
    result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-radius.x, 0.0)) * 2.0;
    result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(radius.x, 0.0)) * 2.0;
    result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, -radius.y)) * 2.0;
    result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, radius.y)) * 2.0;

    // Diagonal directions (low weight)
    result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-radius.x, -radius.y));
    result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(radius.x, -radius.y));
    result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-radius.x, radius.y));
    result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(radius.x, radius.y));

    // Normalize (total weight = 4 + 2*4 + 1*4 = 16)
    return result / 16.0;
}
