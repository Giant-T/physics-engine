struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) center: vec3<f32>,
    @location(1) half_extents: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

// Pas envie d'ecrire ca a la main donc IA
const CUBE_CORNERS: array<vec3<f32>, 8> = array<vec3<f32>, 8>(
    vec3<f32>(-1.0, -1.0, -1.0),
    vec3<f32>( 1.0, -1.0, -1.0),
    vec3<f32>( 1.0,  1.0, -1.0),
    vec3<f32>(-1.0,  1.0, -1.0),

    vec3<f32>(-1.0, -1.0,  1.0),
    vec3<f32>( 1.0, -1.0,  1.0),
    vec3<f32>( 1.0,  1.0,  1.0),
    vec3<f32>(-1.0,  1.0,  1.0),
);

// Pas envie d'ecrire ca a la main donc IA
const CUBE_INDICES: array<u32, 24> = array<u32, 24>(
    // Face arrière
    0, 1, 1, 2,
    2, 3, 3, 0,

    // Face avant
    4, 5, 5, 6,
    6, 7, 7, 4,

    // Connexions entre les faces
    0, 4, 1, 5,
    2, 6, 3, 7,
);

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    let corner = CUBE_CORNERS[CUBE_INDICES[in.vertex_index]];
    let world_pos = corner * in.half_extents + in.center;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
