use std::collections::{HashMap, HashSet};

use nalgebra::{Matrix4, Vector3};
use wgpu::util::DeviceExt;

use crate::{physics::tet_mesh::TetMesh, transform::Transform};

use super::{Renderable, Renderer, pipeline::BasicRenderPipeline, vertex::Vertex};

// TODO: Ajouter une couleur variable
pub struct MeshRenderer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,

    transform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,

    indices: Vec<[u32; 3]>,
    unique_indices: Vec<u32>,
    render_pipeline: BasicRenderPipeline,
}

impl MeshRenderer {
    pub fn new(renderer: &Renderer, vertices: &[Vector3<f64>], indices: &[u32]) -> Self {
        let vertices: Vec<Vertex> = vertices
            .iter()
            .map(|v| Vertex {
                position: [v[0] as f32, v[1] as f32, v[2] as f32],
            })
            .collect();

        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let num_indices = indices.len() as u32;

        let transform_buffer =
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(Matrix4::<f32>::identity().as_slice()),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Model Bind Group Layout"),
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
                });

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Model Bind Group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                }],
            });

        let render_pipeline = BasicRenderPipeline::new(&renderer, &bind_group_layout);

        let indices: Vec<[u32; 3]> = indices
            .chunks_exact(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
            .collect();

        let unique_indices = Self::unique_indices(&indices);

        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
            transform_buffer,
            bind_group,
            indices,
            unique_indices,
            render_pipeline,
        }
    }

    /**
     * Crée un RenderMesh à partir d'un TetMesh
     */
    pub fn from_tet_mesh(tet_mesh: &TetMesh, renderer: &Renderer) -> Self {
        let indices = tet_mesh.surface();
        let unique_indices = Self::unique_indices(&indices);

        let vertices = Self::vertices_from_positions(tet_mesh.rest_positions(), &unique_indices);

        let index_map: HashMap<u32, u32> = unique_indices
            .iter()
            .enumerate()
            .map(|(local, &global)| (global, local as u32))
            .collect();

        let local_indices: Vec<[u32; 3]> = indices
            .iter()
            .map(|&index| {
                [
                    index_map[&index[0]],
                    index_map[&index[1]],
                    index_map[&index[2]],
                ]
            })
            .collect();

        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None, // TODO: Potentiellement ajouter des labels
                contents: bytemuck::cast_slice(&local_indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None, // TODO: Potentiellement ajouter des labels
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let num_indices = indices.len() as u32 * 3;

        let transform_buffer =
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(Matrix4::<f32>::identity().as_slice()),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Model Bind Group Layout"),
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
                });

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Model Bind Group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                }],
            });

        let render_pipeline = BasicRenderPipeline::new(&renderer, &bind_group_layout);

        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
            transform_buffer,
            bind_group,
            render_pipeline,
            indices,
            unique_indices,
        }
    }

    /**
     * Retourne les vertices du tableau de position en se basant sur les indices.
     * Assume que les indices sont uniques et dans l'ordre croissant
     */
    fn vertices_from_positions(
        positions: &[Vector3<f64>],
        unique_indices: &[u32],
    ) -> Vec<[f32; 3]> {
        unique_indices
            .iter()
            .map(|index| positions[*index as usize].map(|x| x as f32).into())
            .collect()
    }

    /**
     * Retourne les indices uniques dans l'ordres croissant
     */
    fn unique_indices(indices: &[[u32; 3]]) -> Vec<u32> {
        let mut seen = HashSet::new();
        let mut unique_indices: Vec<u32> = indices
            .iter()
            .flatten()
            .filter(|&&index| seen.insert(index))
            .map(|&index| index)
            .collect();
        unique_indices.sort();

        unique_indices
    }

    pub fn update_transform_buffer(&self, transform: &Transform<f64>, renderer: &Renderer) {
        renderer.queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::cast_slice(&transform.to_buffer()),
        );
    }

    pub fn update_mesh(&mut self, renderer: &Renderer, current_positions: &[Vector3<f64>]) {
        renderer.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&Self::vertices_from_positions(
                current_positions,
                &self.unique_indices,
            )),
        );
    }
}

impl Renderable for MeshRenderer {
    fn render(&self, renderer: &super::Renderer, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(self.render_pipeline.pipeline());
        render_pass.set_bind_group(0, renderer.camera.bind_group(), &[]);
        render_pass.set_bind_group(1, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
