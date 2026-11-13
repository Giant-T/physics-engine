use nalgebra::Vector3;
use wgpu::util::DeviceExt;

use crate::renderer::{Renderable, Renderer};

use super::{PointInstance, pipeline::PointRenderPipeline};

pub struct PointsRenderer {
    point_buffer: wgpu::Buffer,
    count: u32,
    render_pipeline: PointRenderPipeline,
    capacity: usize,
}

impl PointsRenderer {
    pub fn from_points(points: &[Vector3<f64>], renderer: &Renderer) -> Self {
        let count = points.len() as u32;
        let capacity = count as usize;

        let points: Vec<PointInstance> = points
            .iter()
            .map(|v| PointInstance {
                position: v.map(|x| x as f32).into(),
            })
            .collect();

        let point_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&points),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let render_pipeline = PointRenderPipeline::new(renderer);

        Self {
            render_pipeline,
            point_buffer,
            count,
            capacity,
        }
    }

    pub fn set_points(&mut self, points: &[Vector3<f64>], renderer: &Renderer) {
        self.count = points.len() as u32;

        if self.count as usize > self.capacity {
            self.capacity = self.capacity * 2 + 1;
            self.point_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (self.capacity * size_of::<PointInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        self.count = points.len() as u32;

        let points: Vec<PointInstance> = points
            .iter()
            .map(|v| PointInstance {
                position: v.map(|x| x as f32).into(),
            })
            .collect();

        renderer
            .queue
            .write_buffer(&self.point_buffer, 0, bytemuck::cast_slice(&points));
    }
}

impl Renderable for PointsRenderer {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(self.render_pipeline.pipeline());
        render_pass.set_bind_group(0, renderer.camera.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.point_buffer.slice(..));
        render_pass.draw(0..4, 0..self.count);
    }
}
