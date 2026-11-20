use nalgebra::Vector3;
use wgpu::util::DeviceExt;

use crate::{
    physics::collisions::AABB,
    renderer::{Renderable, Renderer, aabb::AABBInstance},
};

use super::pipeline::AABBRenderPipeline;

pub struct AABBRenderer {
    aabb_buffer: wgpu::Buffer,
    count: usize,
    render_pipeline: AABBRenderPipeline,
    capacity: usize,
}

impl AABBRenderer {
    pub fn from_center_half_extents(
        center: Vector3<f64>,
        half_extents: Vector3<f64>,
        renderer: &Renderer,
    ) -> Self {
        let count = 1;
        let capacity = 1;

        let instance = AABBInstance {
            center: center.map(|x| x as f32).into(),
            half_extents: half_extents.map(|x| x as f32).into(),
        };

        let aabb_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[instance]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let render_pipeline = AABBRenderPipeline::new(renderer);

        Self {
            aabb_buffer,
            count,
            capacity,
            render_pipeline,
        }
    }

    pub fn from_aabb_list(aabbs: &[AABB], renderer: &Renderer) -> AABBRenderer {
        let count = aabbs.len();
        let capacity = count;

        let instances: Box<[AABBInstance]> = aabbs
            .iter()
            .map(|aabb| AABBInstance {
                center: aabb.center.map(|x| x as f32).into(),
                half_extents: aabb.half_extents.map(|x| x as f32).into(),
            })
            .collect();

        let aabb_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let render_pipeline = AABBRenderPipeline::new(renderer);

        Self {
            aabb_buffer,
            count,
            capacity,
            render_pipeline,
        }
    }

    pub fn set_aabbs(&mut self, aabbs: &[AABB], renderer: &Renderer) {
        self.count = aabbs.len();

        if self.count > self.capacity {
            self.capacity = self.capacity * 2 + 1;
            self.aabb_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (self.capacity * size_of::<AABBInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        let instances: Box<[AABBInstance]> = aabbs
            .iter()
            .map(|aabb| AABBInstance {
                center: aabb.center.map(|x| x as f32).into(),
                half_extents: aabb.half_extents.map(|x| x as f32).into(),
            })
            .collect();

        renderer
            .queue
            .write_buffer(&self.aabb_buffer, 0, bytemuck::cast_slice(&instances));
    }
}

impl Renderable for AABBRenderer {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(self.render_pipeline.pipeline());
        render_pass.set_bind_group(0, renderer.camera.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.aabb_buffer.slice(..));
        render_pass.draw(0..24, 0..(self.count as u32));
    }
}
