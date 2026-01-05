// Bloom Downsample Shader
// Performs threshold extraction and downsampling using Kawase dual filtering

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

// Uniform for threshold (only used in first pass)
struct DownsampleUniforms {
    threshold: f32,       // Brightness threshold for bloom extraction
    is_first_pass: u32,   // 1 if this is the threshold pass, 0 otherwise
    _padding1: f32,
    _padding2: f32,
};

@group(0) @binding(2)
var<uniform> uniforms: DownsampleUniforms;

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

    if uniforms.is_first_pass == 1u {
        // Threshold pass: Extract bright pixels
        // Sample 13 taps in a tent pattern for better quality
        let color = textureSample(input_texture, input_sampler, in.uv);

        // Calculate luminance
        let luminance = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));

        // Soft threshold with knee
        let knee = 0.1;
        let threshold = uniforms.threshold;

        var contribution = 0.0;
        if luminance > threshold + knee {
            contribution = 1.0;
        } else if luminance > threshold - knee {
            // Smooth transition in knee region
            let t = (luminance - (threshold - knee)) / (2.0 * knee);
            contribution = smoothstep(0.0, 1.0, t);
        }

        return vec4<f32>(color.rgb * contribution, 1.0);
    } else {
        // Downsample pass: 4-tap box filter (Kawase)
        // Sample in a box pattern at half-pixel offsets
        let offset = texel_size * 0.5;

        var result = vec4<f32>(0.0);
        result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-offset.x, -offset.y));
        result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(offset.x, -offset.y));
        result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-offset.x, offset.y));
        result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(offset.x, offset.y));

        return result * 0.25;
    }
}
