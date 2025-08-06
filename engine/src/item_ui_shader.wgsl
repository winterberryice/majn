// engine/src/item_ui_shader.wgsl

// Vertex shader
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
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
    out.color = model.color;
    out.tex_coords = model.tex_coords;
    return out;
}

// Fragment shader
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Alpha blending check
    if (texture_color.a < 0.1) {
        discard;
    }

    return texture_color * in.color;
}
