// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,    // Kept for potential future use (tinting, etc.)
    @location(2) uv: vec2<f32>,       // Texture coordinates
    @location(3) tree_id: u32,      // Tree ID
    @location(4) light_level: f32,  // Light level (0.0 - 1.0)
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) original_color: vec3<f32>, // Pass original color through
    @location(1) tex_coords: vec2<f32>,   // Pass UVs to fragment shader
    @location(2) tree_id: u32,           // Pass Tree ID to fragment shader
    @location(3) light_level: f32,       // Pass light level to fragment shader
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.original_color = model.color; // Pass through the original vertex color
    out.tex_coords = model.uv;        // Pass through UV coordinates
    out.tree_id = model.tree_id;      // Pass through Tree ID
    out.light_level = model.light_level; // Pass through light level
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
    @location(2) tree_id: u32,
    @location(3) light_level: f32,
}

// Simple hash function to generate a color from a u32
// Source: https://www.shadertoy.com/view/XlGcRh (simplified)
fn hash_to_color(id: u32) -> vec3<f32> {
    var id_float = f32(id);
    let r = fract(sin(id_float * 12.9898) * 43758.5453);
    let g = fract(sin(id_float * 78.233) * 43758.5453);
    let b = fract(sin(id_float * 33.731) * 43758.5453);
    return vec3<f32>(r, g, b);
}


@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    var final_color_rgb: vec3<f32>;
    var final_alpha: f32 = 1.0;

    let sampled_texture_color = textureSample(t_diffuse, s_sampler, in.tex_coords);
    final_alpha = sampled_texture_color.a; // Preserve alpha from texture

    // Sentinel color for Grass Top, set in main.rs: [0.1, 0.9, 0.1]
    let grass_top_sentinel = vec3<f32>(0.1, 0.9, 0.1);
    // Sentinel color for Oak Leaves (used as a fallback or base identifier, not for unique color)
    let oak_leaves_sentinel = vec3<f32>(0.1, 0.9, 0.2);

    // Compare floating point colors with a small epsilon for precision issues
    let grass_color_diff = abs(in.original_color - grass_top_sentinel);
    let is_grass_top = grass_color_diff.x < 0.01 && grass_color_diff.y < 0.01 && grass_color_diff.z < 0.01;

    let leaves_color_diff = abs(in.original_color - oak_leaves_sentinel);
    let is_potential_oak_leaves = leaves_color_diff.x < 0.01 && leaves_color_diff.y < 0.01 && leaves_color_diff.z < 0.01;

    if (is_grass_top) {
        // Texture for grass top is at (0,0) which is grayscale.
        // Use the intensity (e.g., from red channel) from the sampled texture.
        let intensity = sampled_texture_color.r;
        // Apply a greenish tint for grass.
        final_color_rgb = vec3<f32>(intensity * 0.4, intensity * 0.9, intensity * 0.35);
    } else if (is_potential_oak_leaves) { // No longer checking in.tree_id for unique color
        // Texture for oak leaves is at (4,3) which is also grayscale.
        // Use the intensity (e.g., from red channel) from the sampled texture.
        let intensity = sampled_texture_color.r;
        // Apply a standard Minecraft-like oak leaf tint.
        final_color_rgb = vec3<f32>(intensity * 0.25, intensity * 0.55, intensity * 0.15);
    }
    else {
        // For all other blocks/faces, use the sampled texture color directly.
        final_color_rgb = sampled_texture_color.rgb;
    }

    // Apply lighting. Ensure light_level is clamped between 0 and 1 if it isn't already.
    // The problem states it will be normalized to 0.0-1.0 before putting it in the vertex.
    let light_multiplier = clamp(in.light_level, 0.0, 1.0);
    final_color_rgb = final_color_rgb * light_multiplier;

    // A minimum ambient light to prevent pitch black areas where light is 0.
    // This can be adjusted or made configurable.
    let ambient_light_floor = 0.05; // Small amount of light even in darkness
    final_color_rgb = final_color_rgb + vec3<f32>(ambient_light_floor);
    final_color_rgb = clamp(final_color_rgb, vec3<f32>(0.0), vec3<f32>(1.0)); // Ensure color components stay in [0,1]

    return vec4<f32>(final_color_rgb, final_alpha);
}
