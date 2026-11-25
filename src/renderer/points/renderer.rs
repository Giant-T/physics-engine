use nalgebra::Vector3;
use wgpu::util::DeviceExt;

use crate::renderer::{Renderable, Renderer};

use super::{PointInstance, pipeline::PointRenderPipeline};

pub struct PointsRenderer {
    point_buffer: wgpu::Buffer,
    count: usize,
    render_pipeline: PointRenderPipeline,
    bind_group: wgpu::BindGroup,
    color_buffer: wgpu::Buffer,
    capacity: usize,
}

impl PointsRenderer {
    pub fn from_points(points: &[Vector3<f64>], renderer: &Renderer) -> Self {
        let count = points.len();
        let capacity = count;

        let instances: Vec<PointInstance> = points
            .iter()
            .map(|v| PointInstance {
                position: v.map(|x| x as f32).into(),
            })
            .collect();

        let point_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Point Positions Buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let color_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Points Color Buffer"),
                contents: bytemuck::cast_slice(&[1.0_f32, 1.0_f32, 1.0_f32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Point Renderer Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
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
                label: Some("Point Renderer Bind Group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: color_buffer.as_entire_binding(),
                }],
            });

        let render_pipeline = PointRenderPipeline::new(renderer, &bind_group_layout);

        Self {
            render_pipeline,
            point_buffer,
            color_buffer,
            bind_group,
            count,
            capacity,
        }
    }

    pub fn set_color(&self, color: &Vector3<f32>, renderer: &Renderer) {
        renderer.queue.write_buffer(
            &self.color_buffer,
            0,
            bytemuck::cast_slice(color.as_slice()),
        );
    }

    pub fn set_points(&mut self, points: &[Vector3<f64>], renderer: &Renderer) {
        self.count = points.len();

        if self.count > self.capacity {
            self.capacity = self.count;
            self.point_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Point Positions Buffer"),
                size: (self.capacity * size_of::<PointInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        let instances: Vec<PointInstance> = points
            .iter()
            .map(|v| PointInstance {
                position: v.map(|x| x as f32).into(),
            })
            .collect();

        renderer
            .queue
            .write_buffer(&self.point_buffer, 0, bytemuck::cast_slice(&instances));
    }
}

impl Renderable for PointsRenderer {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        if self.count == 0 {
            return;
        }

        render_pass.set_pipeline(self.render_pipeline.pipeline());
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, renderer.camera.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.point_buffer.slice(..));
        render_pass.draw(0..4, 0..(self.count as u32));
    }
}
