use std::rc::Rc;

use wgpu::util::DeviceExt;

use crate::renderer::{Renderer, vertex::Vertex};

use super::pipeline::BasicRenderPipeline;

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32,
    render_pipeline: Rc<BasicRenderPipeline>,
}

impl Mesh {
    pub fn new(
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u16],
        render_pipeline: Rc<BasicRenderPipeline>,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None, // TODO: Potentiellement ajouter des labels
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None, // TODO: Potentiellement ajouter des labels
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_vertices = indices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            num_vertices,
            render_pipeline,
        }
    }

    pub fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(self.render_pipeline.pipeline());
        render_pass.set_bind_group(0, &renderer.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_vertices, 0, 0..1);
    }
}
