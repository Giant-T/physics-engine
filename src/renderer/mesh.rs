use std::rc::Rc;

use wgpu::util::DeviceExt;

use super::{Renderer, pipeline::BasicRenderPipeline, vertex::Vertex};

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    render_pipeline: Rc<BasicRenderPipeline>,
}

impl Mesh {
    pub fn new(
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u32],
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

        let num_indices = indices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
            render_pipeline,
        }
    }

    pub fn load_model(
        path: &str,
        device: &wgpu::Device,
        render_pipeline: Rc<BasicRenderPipeline>,
    ) -> Mesh {
        let obj = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        );
        let (models, _) = obj.expect("Impossible de chargé le fichier OBJ.");
        let model = &models[0];

        let vertices: Vec<Vertex> = (0..model.mesh.positions.len() / 3)
            .map(|i| {
                if model.mesh.normals.is_empty() {
                    Vertex {
                        position: [
                            model.mesh.positions[i * 3],
                            model.mesh.positions[i * 3 + 1],
                            model.mesh.positions[i * 3 + 2],
                        ],
                        normal: [0.0, 0.0, 0.0],
                    }
                } else {
                    Vertex {
                        position: [
                            model.mesh.positions[i * 3],
                            model.mesh.positions[i * 3 + 1],
                            model.mesh.positions[i * 3 + 2],
                        ],
                        normal: [
                            model.mesh.normals[i * 3],
                            model.mesh.normals[i * 3 + 1],
                            model.mesh.normals[i * 3 + 2],
                        ],
                    }
                }
            })
            .collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None, // TODO: Potentiellement ajouter des labels
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None, // TODO: Potentiellement ajouter des labels
            contents: bytemuck::cast_slice(&model.mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = model.mesh.indices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
            render_pipeline,
        }
    }

    pub fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(self.render_pipeline.pipeline());
        render_pass.set_bind_group(0, &renderer.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
