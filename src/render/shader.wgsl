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

// Temperature overlay
@group(2) @binding(0)
var temp_texture: texture_2d<f32>;
@group(2) @binding(1)
var temp_sampler: sampler;

struct OverlayUniform {
    enabled: u32,
    _padding: vec3<u32>,
};

@group(2) @binding(2)
var<uniform> overlay: OverlayUniform;

// Map temperature (in Celsius) to color gradient
fn temperature_to_color(temp: f32) -> vec3<f32> {
    // Temperature ranges and colors:
    // < 0°C: Deep blue (frozen)
    // 0-20°C: Blue to Cyan (cold)
    // 20-50°C: Cyan to Green (cool)
    // 50-100°C: Green to Yellow (warm)
    // 100-200°C: Yellow to Orange (hot)
    // 200-500°C: Orange to Red (very hot)
    // > 500°C: Bright red (extreme)

    if temp < 0.0 {
        return vec3<f32>(0.0, 0.0, 0.5); // Deep blue
    } else if temp < 20.0 {
        let t = temp / 20.0;
        return mix(vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 1.0, 1.0), t); // Blue to cyan
    } else if temp < 50.0 {
        let t = (temp - 20.0) / 30.0;
        return mix(vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(0.0, 1.0, 0.0), t); // Cyan to green
    } else if temp < 100.0 {
        let t = (temp - 50.0) / 50.0;
        return mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 0.0), t); // Green to yellow
    } else if temp < 200.0 {
        let t = (temp - 100.0) / 100.0;
        return mix(vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(1.0, 0.5, 0.0), t); // Yellow to orange
    } else if temp < 500.0 {
        let t = (temp - 200.0) / 300.0;
        return mix(vec3<f32>(1.0, 0.5, 0.0), vec3<f32>(1.0, 0.0, 0.0), t); // Orange to red
    } else {
        return vec3<f32>(1.0, 0.0, 0.0); // Bright red
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Transform screen space to NDC to world space
    let ndc = (in.tex_coords - 0.5) * 2.0;
    let ndc_flipped = vec2<f32>(ndc.x, -ndc.y); // Flip Y for world coordinates

    let world_pos = vec2<f32>(
        (ndc_flipped.x * camera.aspect / camera.zoom) + camera.position.x,
        (ndc_flipped.y / camera.zoom) + camera.position.y
    );

    // Transform world to texture space (512x512, centered at origin)
    let texture_size = 512.0;
    let tex_coords = vec2<f32>(
        (world_pos.x + texture_size * 0.5) / texture_size,
        (world_pos.y + texture_size * 0.5) / texture_size  // No flip - renderer writes Y-up
    );

    // Bounds check
    if tex_coords.x < 0.0 || tex_coords.x > 1.0 ||
       tex_coords.y < 0.0 || tex_coords.y > 1.0 {
        return vec4<f32>(0.1, 0.1, 0.15, 1.0); // Background color
    }

    let color = textureSample(world_texture, world_sampler, tex_coords);

    // Apply temperature overlay if enabled
    if overlay.enabled != 0u {
        // Sample temperature texture (40x40 for 5x5 chunks × 8x8 cells)
        let temp_value = textureSample(temp_texture, temp_sampler, tex_coords).r;
        let temp_color = temperature_to_color(temp_value);

        // Blend with 40% overlay opacity
        let blended = mix(color.rgb, temp_color, 0.4);
        return vec4<f32>(blended * color.a, color.a);
    }

    return vec4<f32>(color.rgb * color.a, color.a);
}
