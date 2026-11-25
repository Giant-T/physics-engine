@group(0) @binding(0)
var<uniform> color: vec3<f32>;

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) vector: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let size = 0.05;

    let world_pos = in.position + select(vec3<f32>(0.0), in.vector, in.vertex_index == 1) * size;

    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(color, 1.0);
}
