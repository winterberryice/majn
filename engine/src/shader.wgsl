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
    @location(3) light_level: f32, // Added light level
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) original_color: vec3<f32>, // Pass original color through
    @location(1) tex_coords: vec2<f32>,   // Pass UVs to fragment shader
    @location(2) v_light_level: f32,    // Pass light level to fragment shader
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.original_color = model.color; // Pass through the original vertex color
    out.tex_coords = model.uv;        // Pass through UV coordinates
    out.v_light_level = model.light_level; // Pass through light level
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
    @location(2) light_level: f32, // Received light level
}

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    var final_sampled_color = textureSample(t_diffuse, s_sampler, in.tex_coords);

    // Check for sentinel color indicating Grass Top that needs tinting
    // Sentinel color for Grass Top, set in main.rs: [0.1, 0.9, 0.1]
    let grass_top_sentinel = vec3<f32>(0.1, 0.9, 0.1);
    // Sentinel color for Oak Leaves, set in main.rs: [0.1, 0.9, 0.2]
    let oak_leaves_sentinel = vec3<f32>(0.1, 0.9, 0.2);

    // Compare floating point colors with a small epsilon for precision issues
    let grass_color_diff = abs(in.original_color - grass_top_sentinel);
    let is_grass_top = grass_color_diff.x < 0.01 && grass_color_diff.y < 0.01 && grass_color_diff.z < 0.01;

    let leaves_color_diff = abs(in.original_color - oak_leaves_sentinel);
    let is_oak_leaves = leaves_color_diff.x < 0.01 && leaves_color_diff.y < 0.01 && leaves_color_diff.z < 0.01;

    if (is_grass_top || is_oak_leaves) {
        // Texture for grass top is at (0,0) which is grayscale.
        // Texture for oak leaves is at (4,3) which is also grayscale.
        // Use the intensity (e.g., from red channel) from the sampled texture.
        let intensity = final_sampled_color.r;
        // Apply a greenish tint. Adjust factors for desired green hue.
        // Example: (R: low, G: high, B: medium-low)
        // Using the same tint factors for both grass and leaves as per requirement "similar tint".
        let tinted_color = vec3<f32>(intensity * 0.4, intensity * 0.9, intensity * 0.35);
        final_sampled_color = vec4<f32>(tinted_color, final_sampled_color.a);
    }

    // Apply lighting
    // Ensure light level is at least a minimum ambient value to avoid pure black.
    let ambient_light = 0.05; // Smallest light contribution
    let effective_light = max(in.light_level, ambient_light);
    let lit_color = final_sampled_color.rgb * effective_light;

    return vec4<f32>(lit_color, final_sampled_color.a);
}
