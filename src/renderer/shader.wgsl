struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(0) @binding(1)
var<uniform> camera_position: vec3<f32>;

@group(1) @binding(0)
var<uniform> light_position: vec3<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) to_viewer: vec3<f32>,
    @location(2) to_light: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);

    out.to_viewer = camera_position - in.position;
    out.to_light = light_position - in.position;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = vec3<f32>(0.941, 0.643, 0.063);

    let to_viewer = normalize(in.to_viewer);
    let to_light = normalize(in.to_light);

    let depth = in.clip_position.z / in.clip_position.w;
    let near = 0.1;
    let far = 100.0;
    let factor = smoothstep(near, far, depth);

    return vec4<f32>(
        color / depth,
        1.0
    );
}
