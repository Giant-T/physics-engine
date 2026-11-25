use std::mem::size_of;

mod pipeline;
mod renderer;

pub use renderer::VectorsRenderer;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VectorInstance {
    pub position: [f32; 3],
    pub vector: [f32; 3],
}

const INSTANCE_ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
    0 => Float32x3,
    1 => Float32x3,
];

impl VectorInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<VectorInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &INSTANCE_ATTRIBUTES,
        }
    }
}
