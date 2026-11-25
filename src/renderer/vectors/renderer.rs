use nalgebra::Vector3;
use wgpu::util::DeviceExt;

use crate::renderer::{
    Renderable, Renderer,
    vectors::{VectorInstance, pipeline::VectorRenderPipeline},
};

pub struct VectorsRenderer {
    vector_buffer: wgpu::Buffer,
    count: usize,
    capacity: usize,
    color_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    render_pipeline: VectorRenderPipeline,
}

impl VectorsRenderer {
    pub fn from_vectors_positions(
        vectors_and_positions: &[(Vector3<f64>, Vector3<f64>)],
        renderer: &Renderer,
    ) -> Self {
        let count = vectors_and_positions.len();
        let capacity = count;

        let instances: Vec<VectorInstance> = vectors_and_positions
            .iter()
            .map(|(v, p)| VectorInstance {
                position: p.map(|x| x as f32).into(),
                vector: v.map(|x| x as f32).into(),
            })
            .collect();

        let vector_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let color_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vector Color Buffer"),
                contents: bytemuck::cast_slice(&[1.0_f32, 1.0_f32, 1.0f32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Vector Renderer Bind Group Layout"),
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
                label: Some("Vector Renderer Bind Group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: color_buffer.as_entire_binding(),
                }],
            });

        let render_pipeline = VectorRenderPipeline::new(renderer, &bind_group_layout);

        Self {
            count,
            capacity,
            vector_buffer,
            color_buffer,
            bind_group,
            render_pipeline,
        }
    }

    pub fn set_color(&self, color: &Vector3<f32>, renderer: &Renderer) {
        renderer.queue.write_buffer(
            &self.color_buffer,
            0,
            bytemuck::cast_slice(color.as_slice()),
        );
    }

    pub fn set_vectors_positions(
        &mut self,
        vectors_and_positions: &[(Vector3<f64>, Vector3<f64>)],
        renderer: &Renderer,
    ) {
        self.count = vectors_and_positions.len();

        if self.count > self.capacity {
            self.capacity = self.count;
            self.vector_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Point Positions Buffer"),
                size: (self.capacity * size_of::<VectorInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        let instances: Vec<VectorInstance> = vectors_and_positions
            .iter()
            .map(|(v, p)| VectorInstance {
                vector: v.map(|x| x as f32).into(),
                position: p.map(|x| x as f32).into(),
            })
            .collect();

        renderer
            .queue
            .write_buffer(&self.vector_buffer, 0, bytemuck::cast_slice(&instances));
    }
}

impl Renderable for VectorsRenderer {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        if self.count == 0 {
            return;
        }

        render_pass.set_pipeline(self.render_pipeline.pipeline());
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, renderer.camera.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.vector_buffer.slice(..));
        render_pass.draw(0..2, 0..(self.count as u32));
    }
}
