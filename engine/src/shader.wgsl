// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>, // Now world position
    @location(1) color: vec3<f32>,    // Vertex color (block type color)
};

// InstanceInput is no longer used for blocks

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>, // Color to pass to fragment shader
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput { // No instance input
    var out: VertexOutput;
    // Vertex position is already in world space (or chunk-local world space).
    // Transform directly by view_proj matrix.
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    // Color comes directly from the vertex data.
    out.color = model.color;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
