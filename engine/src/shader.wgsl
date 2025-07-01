// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,    // For tinting sentinels
    @location(2) uv: vec2<f32>,       // Texture coordinates
    @location(3) light_level: f32,  // Normalized light level (0.0 to 1.0)
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) original_color: vec3<f32>, // Pass original color through for tinting logic
    @location(1) tex_coords: vec2<f32>,   // Pass UVs to fragment shader
    @location(2) light_level: f32,      // Pass light level to fragment shader
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.original_color = model.color;
    out.tex_coords = model.uv;
    out.light_level = model.light_level; // Pass light level
    return out;
}

// Fragment shader

// Texture and Sampler bindings
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_sampler: sampler;

// Input from vertex shader (matches VertexOutput)
struct FragmentInput {
    @location(0) original_color: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) light_level: f32,
}

const MIN_AMBIENT_LIGHT: f32 = 0.05; // A small amount of light even in darkness to see shapes

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    var final_sampled_color = textureSample(t_diffuse, s_sampler, in.tex_coords);

    // Sentinel color for Grass Top: [0.1, 0.9, 0.1]
    let grass_top_sentinel = vec3<f32>(0.1, 0.9, 0.1);
    // Sentinel color for Oak Leaves: [0.1, 0.9, 0.2]
    let oak_leaves_sentinel = vec3<f32>(0.1, 0.9, 0.2);

    let grass_color_diff = abs(in.original_color - grass_top_sentinel);
    let is_grass_top = grass_color_diff.x < 0.01 && grass_color_diff.y < 0.01 && grass_color_diff.z < 0.01;

    let leaves_color_diff = abs(in.original_color - oak_leaves_sentinel);
    let is_oak_leaves = leaves_color_diff.x < 0.01 && leaves_color_diff.y < 0.01 && leaves_color_diff.z < 0.01;

    if (is_grass_top || is_oak_leaves) {
        let intensity = final_sampled_color.r;
        let tinted_color = vec3<f32>(intensity * 0.4, intensity * 0.9, intensity * 0.35);
        final_sampled_color = vec4<f32>(tinted_color, final_sampled_color.a);
    }

    // Apply lighting
    // Ensure there's always a minimum light level so blocks are not pitch black
    let effective_light = max(in.light_level, MIN_AMBIENT_LIGHT);
    let lit_color = final_sampled_color.rgb * effective_light;

    return vec4<f32>(lit_color, final_sampled_color.a);
}
