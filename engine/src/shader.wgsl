// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tree_id: u32,
    @location(4) light: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) original_color: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) tree_id: u32,
    @location(3) light_level: vec2<f32>, // Pass unpacked light to fragment shader
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.original_color = model.color;
    out.tex_coords = model.uv;
    out.tree_id = model.tree_id;

    // Unpack the light levels and pass them to the fragment shader
    // We store sun_light in .x and block_light in .y
    let sun_light = f32(model.light & 0xFFu);
    let block_light = f32((model.light >> 8u) & 0xFFu);
    out.light_level = vec2<f32>(sun_light, block_light);

    return out;
}

// Fragment shader

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_sampler: sampler;

// Input from vertex shader (matches VertexOutput)
struct FragmentInput {
    @location(0) original_color: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) tree_id: u32,
    @location(3) light_level: vec2<f32>, // New! sun_light is in .x, block_light in .y
}

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    let sampled_color = textureSample(t_diffuse, s_sampler, in.tex_coords);
    var final_color = sampled_color.rgb;

    // --- Biome tinting logic (your existing code) ---
    let grass_top_sentinel = vec3<f32>(0.1, 0.9, 0.1);
    let oak_leaves_sentinel = vec3<f32>(0.1, 0.9, 0.2);

    let grass_color_diff = abs(in.original_color - grass_top_sentinel);
    let is_grass_top = grass_color_diff.x < 0.01 && grass_color_diff.y < 0.01 && grass_color_diff.z < 0.01;

    let leaves_color_diff = abs(in.original_color - oak_leaves_sentinel);
    let is_potential_oak_leaves = leaves_color_diff.x < 0.01 && leaves_color_diff.y < 0.01 && leaves_color_diff.z < 0.01;

    if (is_grass_top) {
        let intensity = sampled_color.r;
        final_color = vec3<f32>(intensity * 0.4, intensity * 0.9, intensity * 0.35);
    } else if (is_potential_oak_leaves) {
        let intensity = sampled_color.r;
        final_color = vec3<f32>(intensity * 0.25, intensity * 0.55, intensity * 0.15);
    }
    // --- End of biome tinting logic ---

    // --- NEW LIGHTING CALCULATION ---
    let sun_light = in.light_level.x;
    let block_light = in.light_level.y;

    // Use the brightest of the two light sources for this pixel
    let brightest_light = max(sun_light, block_light);

    // Convert the 0-15 light level to a 0.0-1.0 brightness multiplier.
    // We use pow() to make the light fall off more naturally (less linear).
    // We also add a minimum ambient light so caves aren't pitch black.
    let ambient_light = 0.2;
    let light_multiplier = pow(brightest_light / 15.0, 1.5);
    let final_light = max(light_multiplier, ambient_light);

    // Apply the light multiplier to the final color
    final_color = final_color * final_light;
    // --- END of new lighting calculation ---

    return vec4<f32>(final_color, sampled_color.a);
}
