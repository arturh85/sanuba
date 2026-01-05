// Vertex shader

struct CameraUniform {
    position: vec2<f32>,
    zoom: f32,
    aspect: f32,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

// Texture origin for dynamic camera-centered rendering
@group(1) @binding(1)
var<uniform> texture_origin: vec2<f32>;

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

// Debug overlays (temperature and light)
@group(2) @binding(0)
var temp_texture: texture_2d<f32>;
@group(2) @binding(1)
var temp_sampler: sampler;
@group(2) @binding(2)
var light_texture: texture_2d<f32>;
@group(2) @binding(3)
var light_sampler: sampler;

struct OverlayUniform {
    overlay_type: u32,  // 0=none, 1=temperature, 2=light
    _padding: vec3<u32>,
};

@group(2) @binding(4)
var<uniform> overlay: OverlayUniform;

// Post-processing uniform
struct PostProcessUniform {
    time: f32,
    scanline_intensity: f32,   // 0.0 = off, 0.1-0.3 = subtle
    vignette_intensity: f32,   // 0.0 = off, 0.3-0.5 = noticeable
    bloom_intensity: f32,      // 0.0 = off, 0.2-0.5 = noticeable
    // Water noise animation parameters
    water_noise_frequency: f32,
    water_noise_speed: f32,
    water_noise_amplitude: f32,
    // Lava noise animation parameters
    lava_noise_frequency: f32,
    lava_noise_speed: f32,
    lava_noise_amplitude: f32,
};

@group(2) @binding(5)
var<uniform> post_process: PostProcessUniform;

// Bloom texture (multi-pass bloom result)
@group(2) @binding(6)
var bloom_texture: texture_2d<f32>;
@group(2) @binding(7)
var bloom_sampler: sampler;

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

// Apply scanline effect (retro CRT look)
fn apply_scanlines(color: vec3<f32>, screen_y: f32, intensity: f32) -> vec3<f32> {
    if intensity <= 0.0 {
        return color;
    }
    // Create horizontal scanline pattern - every 2-3 pixels for visibility
    let scanline = sin(screen_y * 0.5 * 3.14159) * 0.5 + 0.5;
    let darkening = 1.0 - intensity * (1.0 - scanline);
    return color * darkening;
}

// Apply vignette effect (darkened edges)
fn apply_vignette(color: vec3<f32>, uv: vec2<f32>, intensity: f32) -> vec3<f32> {
    if intensity <= 0.0 {
        return color;
    }
    // Calculate distance from center
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(uv, center);
    // Smooth falloff from center to edges
    let vignette = 1.0 - smoothstep(0.3, 0.9, dist) * intensity;
    return color * vignette;
}

// Apply fake bloom/glow effect for bright pixels
fn apply_bloom(color: vec3<f32>, intensity: f32) -> vec3<f32> {
    if intensity <= 0.0 {
        return color;
    }
    // Calculate luminance
    let luminance = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    // Boost bright pixels
    let bloom_factor = smoothstep(0.6, 1.0, luminance);
    let boosted = color * (1.0 + bloom_factor * intensity);
    return min(boosted, vec3<f32>(1.0, 1.0, 1.0)); // Clamp to prevent overflow
}

// 3D Simplex-like noise function for water/lava animation
// Based on WebGL/WGSL implementations of simplex noise
fn mod289_v3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_v4(x: vec4<f32>) -> vec4<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute_v4(x: vec4<f32>) -> vec4<f32> {
    return mod289_v4(((x * 34.0) + 1.0) * x);
}

fn taylorInvSqrt_v4(r: vec4<f32>) -> vec4<f32> {
    return 1.79284291400159 - 0.85373472095314 * r;
}

fn simplex3D(v: vec3<f32>) -> f32 {
    let C = vec2<f32>(1.0/6.0, 1.0/3.0);
    let D = vec4<f32>(0.0, 0.5, 1.0, 2.0);

    // First corner
    var i  = floor(v + dot(v, C.yyy));
    let x0 = v - i + dot(i, C.xxx);

    // Other corners
    let g = step(x0.yzx, x0.xyz);
    let l = 1.0 - g;
    let i1 = min(g.xyz, l.zxy);
    let i2 = max(g.xyz, l.zxy);

    let x1 = x0 - i1 + C.xxx;
    let x2 = x0 - i2 + C.yyy;
    let x3 = x0 - D.yyy;

    // Permutations
    i = mod289_v3(i);
    let p = permute_v4(permute_v4(permute_v4(
        i.z + vec4<f32>(0.0, i1.z, i2.z, 1.0))
        + i.y + vec4<f32>(0.0, i1.y, i2.y, 1.0))
        + i.x + vec4<f32>(0.0, i1.x, i2.x, 1.0));

    // Gradients
    let n_ = 0.142857142857; // 1.0/7.0
    let ns = n_ * D.wyz - D.xzx;

    let j = p - 49.0 * floor(p * ns.z * ns.z);

    let x_ = floor(j * ns.z);
    let y_ = floor(j - 7.0 * x_);

    let x = x_ * ns.x + ns.yyyy;
    let y = y_ * ns.x + ns.yyyy;
    let h = 1.0 - abs(x) - abs(y);

    let b0 = vec4<f32>(x.xy, y.xy);
    let b1 = vec4<f32>(x.zw, y.zw);

    let s0 = floor(b0) * 2.0 + 1.0;
    let s1 = floor(b1) * 2.0 + 1.0;
    let sh = -step(h, vec4<f32>(0.0));

    let a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    let a1 = b1.xzyw + s1.xzyw * sh.zzww;

    var p0 = vec3<f32>(a0.xy, h.x);
    var p1 = vec3<f32>(a0.zw, h.y);
    var p2 = vec3<f32>(a1.xy, h.z);
    var p3 = vec3<f32>(a1.zw, h.w);

    // Normalize gradients
    let norm = taylorInvSqrt_v4(vec4<f32>(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 = p0 * norm.x;
    p1 = p1 * norm.y;
    p2 = p2 * norm.z;
    p3 = p3 * norm.w;

    // Mix final noise value
    var m = max(0.6 - vec4<f32>(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3)), vec4<f32>(0.0));
    m = m * m;
    return 42.0 * dot(m * m, vec4<f32>(dot(p0, x0), dot(p1, x1), dot(p2, x2), dot(p3, x3)));
}

// Map light level (0-15) to color gradient
fn light_to_color(light_level: f32) -> vec3<f32> {
    // Light levels (0-15):
    // 0: Complete darkness (black)
    // 1-3: Very dark (deep purple/blue)
    // 4-7: Dim (purple to blue)
    // 8-11: Moderate (blue to cyan)
    // 12-14: Bright (cyan to white)
    // 15: Full light (bright white)

    let normalized = clamp(light_level / 15.0, 0.0, 1.0);

    if normalized < 0.2 {
        // 0-3: Black to deep purple
        let t = normalized / 0.2;
        return mix(vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.2, 0.0, 0.4), t);
    } else if normalized < 0.5 {
        // 4-7: Deep purple to blue
        let t = (normalized - 0.2) / 0.3;
        return mix(vec3<f32>(0.2, 0.0, 0.4), vec3<f32>(0.0, 0.3, 0.8), t);
    } else if normalized < 0.75 {
        // 8-11: Blue to cyan
        let t = (normalized - 0.5) / 0.25;
        return mix(vec3<f32>(0.0, 0.3, 0.8), vec3<f32>(0.0, 0.8, 1.0), t);
    } else {
        // 12-15: Cyan to bright white
        let t = (normalized - 0.75) / 0.25;
        return mix(vec3<f32>(0.0, 0.8, 1.0), vec3<f32>(1.0, 1.0, 1.0), t);
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

    // Transform world to texture space using dynamic texture origin
    let texture_size = 2048.0;
    let relative_pos = world_pos - texture_origin;
    let tex_coords = relative_pos / texture_size;

    // Bounds check - clamp to valid texture coordinates
    if tex_coords.x < 0.0 || tex_coords.x > 1.0 ||
       tex_coords.y < 0.0 || tex_coords.y > 1.0 {
        // Apply post-processing even to background
        var bg_color = vec3<f32>(0.1, 0.1, 0.15);
        bg_color = apply_vignette(bg_color, in.tex_coords, post_process.vignette_intensity);
        bg_color = apply_scanlines(bg_color, in.clip_position.y, post_process.scanline_intensity);
        return vec4<f32>(bg_color, 1.0);
    }

    let color = textureSample(world_texture, world_sampler, tex_coords);
    var final_color = color.rgb;

    // Apply procedural noise animation to water and lava
    // Detect water by color signature (blue-ish with low red)
    let is_water = final_color.b > 0.5 && final_color.r < 0.4 && final_color.g < 0.7;
    // Detect lava by color signature (red-orange)
    let is_lava = final_color.r > 0.7 && final_color.g > 0.3 && final_color.g < 0.6 && final_color.b < 0.3;

    if is_water {
        // Animate water with flowing turbulence
        let noise_input = vec3<f32>(
            world_pos.x * post_process.water_noise_frequency,
            world_pos.y * post_process.water_noise_frequency,
            post_process.time * post_process.water_noise_speed
        );
        let noise = simplex3D(noise_input);
        // Apply noise as color variation (creates flowing effect)
        final_color = final_color + vec3<f32>(noise * post_process.water_noise_amplitude);
        final_color = clamp(final_color, vec3<f32>(0.0), vec3<f32>(1.0));
    } else if is_lava {
        // Animate lava with bubbling/pulsing effect
        let noise_input = vec3<f32>(
            world_pos.x * post_process.lava_noise_frequency,
            world_pos.y * post_process.lava_noise_frequency,
            post_process.time * post_process.lava_noise_speed
        );
        let noise = simplex3D(noise_input);
        // Convert noise to glow (always positive, 0-1 range)
        let glow = (noise * 0.5 + 0.5) * post_process.lava_noise_amplitude;
        // Add orange-red glow to lava
        final_color = final_color + vec3<f32>(glow, glow * 0.6, 0.0);
        final_color = clamp(final_color, vec3<f32>(0.0), vec3<f32>(1.0));
    }

    // Apply debug overlays
    if overlay.overlay_type == 1u {
        // Temperature overlay - snap camera to chunk boundaries to match CPU-side texture updates
        let chunk_size = 64.0;
        let snapped_cam_x = floor(camera.position.x / chunk_size) * chunk_size;
        let snapped_cam_y = floor(camera.position.y / chunk_size) * chunk_size;
        let temp_texture_size = 320.0;  // 5 chunks × 64 pixels
        let temp_tex_coords = vec2<f32>(
            (world_pos.x - snapped_cam_x + temp_texture_size * 0.5) / temp_texture_size,
            (world_pos.y - snapped_cam_y + temp_texture_size * 0.5) / temp_texture_size
        );
        let temp_value = textureSample(temp_texture, temp_sampler, temp_tex_coords).r;
        let temp_color = temperature_to_color(temp_value);
        final_color = mix(final_color, temp_color, 0.4);
    } else if overlay.overlay_type == 2u {
        // Light overlay - snap camera to chunk boundaries to match CPU-side texture updates
        let chunk_size = 64.0;
        let snapped_cam_x = floor(camera.position.x / chunk_size) * chunk_size;
        let snapped_cam_y = floor(camera.position.y / chunk_size) * chunk_size;
        let light_texture_size = 320.0;  // 5 chunks × 64 pixels
        let light_tex_coords = vec2<f32>(
            (world_pos.x - snapped_cam_x + light_texture_size * 0.5) / light_texture_size,
            (world_pos.y - snapped_cam_y + light_texture_size * 0.5) / light_texture_size
        );
        let light_value = textureSample(light_texture, light_sampler, light_tex_coords).r;
        let light_color = light_to_color(light_value);
        final_color = mix(final_color, light_color, 0.5);
    }

    // Apply post-processing effects

    // Apply multi-pass bloom (sample from bloom texture and blend)
    let bloom_color = textureSample(bloom_texture, bloom_sampler, in.tex_coords).rgb;
    final_color = final_color + bloom_color * post_process.bloom_intensity;

    // Apply other post-processing effects
    final_color = apply_vignette(final_color, in.tex_coords, post_process.vignette_intensity);
    final_color = apply_scanlines(final_color, in.clip_position.y, post_process.scanline_intensity);

    return vec4<f32>(final_color * color.a, color.a);
}
