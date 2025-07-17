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
    @location(4) sky_light: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) original_color: vec3<f32>, // Pass original color through
    @location(1) tex_coords: vec2<f32>,   // Pass UVs to fragment shader
    @location(2) tree_id: u32,           // Pass Tree ID to fragment shader
    @location(3) @interpolate(flat) sky_light: u32,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.original_color = model.color; // Pass through the original vertex color
    out.tex_coords = model.uv;        // Pass through UV coordinates
    out.tree_id = model.tree_id;      // Pass through Tree ID
    out.sky_light = model.sky_light;
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
    @location(3) @interpolate(flat) sky_light: u32,
}

// Calculates a point on a 1D cubic Bezier curve.
// t: The input progress (0.0 to 1.0)
// p0, p3: The start and end values of the curve
// p1, p2: The control point "handles" that shape the curve
fn cubic_bezier(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t_inv = 1.0 - t;
    let t_inv_sq = t_inv * t_inv;
    let t_sq = t * t;

    return (t_inv_sq * t_inv * p0) +
           (3.0 * t_inv_sq * t * p1) +
           (3.0 * t_inv * t_sq * p2) +
           (t_sq * t * p3);
}

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    let sampled_color = textureSample(t_diffuse, s_sampler, in.tex_coords);

    // Alpha Testing: If the pixel's alpha is very low, discard it completely.
    if (sampled_color.a < 0.1) {
        discard;
    }

    let normalized_light = f32(in.sky_light) / 15.0;

    // 1. Calculate the curve's output (from 0.0 to 1.0) using control points
    //    that create a sharp "ease-in" effect.
    let bezier_y = cubic_bezier(0.0, 0.0, 0.1, 1.0, normalized_light);

    // 2. Remap the curve's output to your desired brightness range [0.05, 1.0].
    let light_intensity = 0.05 + bezier_y * 0.95;

    var final_color = sampled_color.rgb * light_intensity;

    // Sentinel color for Grass Top, set in main.rs: [0.1, 0.9, 0.1]
    let grass_top_sentinel = vec3<f32>(0.1, 0.9, 0.1);
    // Sentinel color for Oak Leaves
    let oak_leaves_sentinel = vec3<f32>(0.1, 0.9, 0.2);

    // Compare floating point colors with a small epsilon for precision issues
    let grass_color_diff = abs(in.original_color - grass_top_sentinel);
    let is_grass_top = grass_color_diff.x < 0.01 && grass_color_diff.y < 0.01 && grass_color_diff.z < 0.01;

    let leaves_color_diff = abs(in.original_color - oak_leaves_sentinel);
    let is_potential_oak_leaves = leaves_color_diff.x < 0.01 && leaves_color_diff.y < 0.01 && leaves_color_diff.z < 0.01;

    if (is_grass_top) {
        let intensity = sampled_color.r;
        let tinted_color = vec3<f32>(intensity * 0.4, intensity * 0.9, intensity * 0.35);
        final_color = tinted_color * light_intensity;
    } else if (is_potential_oak_leaves) {
        let intensity = sampled_color.r;
        let oak_tinted_color = vec3<f32>(intensity * 0.25, intensity * 0.55, intensity * 0.15);
        final_color = oak_tinted_color * light_intensity;
    }

    return vec4<f32>(final_color.rgb, 1.0); // Return full alpha
}
