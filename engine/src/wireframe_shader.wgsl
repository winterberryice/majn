// Basic Wireframe Shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct ModelUniform {
    model: mat4x4<f32>,
}
@group(1) @binding(0) // New bind group for model matrix
var<uniform> model_uniform: ModelUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    model_in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // Apply model matrix first, then view_proj
    out.clip_position = camera.view_proj * model_uniform.model * vec4<f32>(model_in.position, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black color for the wireframe
}
