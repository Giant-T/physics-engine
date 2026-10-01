use wgpu::util::DeviceExt;

use super::{Renderer, mesh::Mesh, pipeline::BasicRenderPipeline, transform::Transform};

pub struct RenderObject {
    transform: Transform,
    mesh: Mesh,
    render_pipeline: BasicRenderPipeline,

    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl RenderObject {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
        transform: Transform,
        mesh: Mesh,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[transform.to_buffer()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        let mut bind_group_layouts = Vec::from(bind_group_layouts);
        bind_group_layouts.push(Some(&bind_group_layout));
        let render_pipeline = BasicRenderPipeline::new(&device, config, &bind_group_layouts);

        Self {
            transform,
            mesh,
            render_pipeline,
            buffer,
            bind_group,
        }
    }

    /*
     * TODO: update de transform
     * pub fn update(&mut self, device: &wgpu::Device) {}
     */

    pub fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(self.render_pipeline.pipeline());
        render_pass.set_bind_group(0, renderer.camera.bind_group(), &[]);
        render_pass.set_bind_group(1, renderer.light.bind_group(), &[]);
        render_pass.set_bind_group(2, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.mesh.num_indices, 0, 0..1);
    }
}
