// engine/src/ui_shader.wgsl

// Vertex shader
struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct ProjectionUniform {
    projection_matrix: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> u_projection: ProjectionUniform;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = u_projection.projection_matrix * vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.9, 0.9, 0.9, 0.75); // Semi-transparent light grey
}
