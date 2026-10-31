use std::collections::{HashMap, HashSet};

use nalgebra::Vector3;
use wgpu::util::DeviceExt;

use crate::physics::tet_mesh::TetMesh;

use super::{Renderable, Renderer, pipeline::BasicRenderPipeline, vertex::Vertex};

pub struct MeshRenderer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,

    indices: Vec<[u32; 3]>,
    unique_indices: Vec<u32>,
    render_pipeline: BasicRenderPipeline,
}

impl MeshRenderer {
    pub fn new(renderer: &Renderer, vertices: &[Vertex], indices: &[u32]) -> Self {
        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None, // TODO: Potentiellement ajouter des labels
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None, // TODO: Potentiellement ajouter des labels
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let num_indices = indices.len() as u32;

        let render_pipeline = BasicRenderPipeline::new(&renderer);

        let indices: Vec<[u32; 3]> = indices
            .chunks_exact(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
            .collect();

        let unique_indices = Self::unique_indices(&indices);

        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
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
                usage: wgpu::BufferUsages::VERTEX,
            });

        let num_indices = indices.len() as u32 * 3;

        let render_pipeline = BasicRenderPipeline::new(renderer);

        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
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
}

impl Renderable for MeshRenderer {
    fn render(&self, renderer: &super::Renderer, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(self.render_pipeline.pipeline());
        render_pass.set_bind_group(0, renderer.camera.bind_group(), &[]);
        render_pass.set_bind_group(1, renderer.light.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
