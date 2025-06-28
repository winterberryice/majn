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
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) original_color: vec3<f32>, // Pass original color through
    @location(1) tex_coords: vec2<f32>,   // Pass UVs to fragment shader
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.original_color = model.color; // Pass through the original vertex color
    out.tex_coords = model.uv;        // Pass through UV coordinates
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
}

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    // Sample the texture
    let texture_color = textureSample(t_diffuse, s_sampler, in.tex_coords);

    // For now, just use the texture color.
    // We could multiply by in.original_color for tinting: texture_color * vec4<f32>(in.original_color, 1.0)
    return texture_color;
}
