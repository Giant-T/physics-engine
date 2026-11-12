struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(0) @binding(2)
var<uniform> camera_up: vec3<f32>;
@group(0) @binding(3)
var<uniform> camera_right: vec3<f32>;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

const QUAD: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 1.0,  1.0),
);

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let corner = QUAD[in.vertex_index];
    let size = 0.01;

    let world_pos = in.position
        + camera_right * corner.x * size
        + camera_up * corner.y * size;

    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.uv = corner;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (length(in.uv) > 1.0) {
        discard;
    }

    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
