// This is WGSL, WGPU's Shading Language. It's similar to GLSL but with Rust-like syntax.

// This struct defines the input to our vertex shader.
// It matches the `Vertex` struct in our Rust code.
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

// This struct defines the output of our vertex shader,
// which becomes the input to our fragment shader.
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

// Vertex Shader
// This function runs for every vertex in our triangle.
@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0); // Pass the position through
    out.color = model.color; // Pass the color through
    return out;
}

// Fragment Shader
// This function runs for every pixel of our triangle.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Return the color passed from the vertex shader.
    // The GPU automatically interpolates the color between the vertices.
    return vec4<f32>(in.color, 1.0);
}