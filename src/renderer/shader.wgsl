struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(0) @binding(1)
var<uniform> camera_position: vec3<f32>;

@group(1) @binding(0)
var<uniform> light_position: vec3<f32>;

@group(2) @binding(0)
var<uniform> model: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) to_viewer: vec3<f32>,
    @location(2) to_light: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let world_position = (model * vec4<f32>(in.position, 1.0)).xyz;
    var out: VertexOutput;

    out.clip_position = camera.view_proj * model * vec4<f32>(in.position, 1.0);
    out.normal = normalize(in.normal);

    out.to_viewer = camera_position - world_position;
    out.to_light = light_position - world_position;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let diffuse_color = vec3<f32>(0.941, 0.643, 0.063);

    let to_viewer = normalize(in.to_viewer);
    let normal = normalize(in.normal);
    let to_light = normalize(in.to_light);

    let diffuse_intensity = max(0.1, dot(to_light, normal));

    return vec4<f32>(
        diffuse_color * diffuse_intensity,
        1.0
    );
}
