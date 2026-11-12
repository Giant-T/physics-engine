use nalgebra::Vector3;
use wgpu::util::DeviceExt;

use crate::renderer::{Renderable, Renderer};

use super::{PinInstance, pipeline::PinRenderPipeline};

pub struct PinsRenderer {
    pin_buffer: wgpu::Buffer,
    count: u32,
    render_pipeline: PinRenderPipeline,
}

impl PinsRenderer {
    pub fn from_pins(pins: &[Vector3<f64>], renderer: &Renderer) -> Self {
        let count = pins.len() as u32;

        let pins: Vec<PinInstance> = pins
            .iter()
            .map(|v| PinInstance {
                position: v.map(|x| x as f32).into(),
            })
            .collect();

        log::info!("{pins:?}");

        let pin_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&pins),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let render_pipeline = PinRenderPipeline::new(renderer);

        Self {
            render_pipeline,
            pin_buffer,
            count,
        }
    }
}

impl Renderable for PinsRenderer {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(self.render_pipeline.pipeline());
        render_pass.set_bind_group(0, renderer.camera.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.pin_buffer.slice(..));
        render_pass.draw(0..4, 0..self.count);
    }
}
