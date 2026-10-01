use std::sync::Arc;

use cgmath::{Point3, Quaternion, Rotation3, Vector3, num_traits::One};
use winit::window::Window;

use camera::Camera;

mod camera;
mod light;
mod mesh;
mod object;
mod pipeline;
mod texture;
mod transform;
mod vertex;

use light::PointLight;
use mesh::Mesh;
use texture::Texture;

pub use camera::CameraController;
pub use object::RenderObject;
pub use transform::Transform;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

pub struct Renderer {
    is_surface_configured: bool,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    render_objects: Box<[RenderObject]>,
    light: PointLight,

    depth_texture: Texture,

    pub camera: Camera,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            display: None,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: Default::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: Default::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        let camera = Camera::new(
            &device,
            Transform::from_position_and_rotation(Point3::new(0.0, 0.0, 5.0), Quaternion::one()),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
        );

        let depth_texture = Texture::create_depth_texture(&device, &config);
        let light = PointLight::new(
            &device,
            Transform::from_translation(Vector3::new(0.0, 30.0, 0.0)),
        );

        let render_objects = Box::new([
            RenderObject::new(
                &device,
                &config,
                &[
                    Some(camera.bind_group_layout()),
                    Some(light.bind_group_layout()),
                ],
                Transform::from_rotation(Quaternion::from_angle_y(cgmath::Deg(90.0))),
                Mesh::load_model("stanford-bunny.obj", &device),
            ),
            RenderObject::new(
                &device,
                &config,
                &[
                    Some(camera.bind_group_layout()),
                    Some(light.bind_group_layout()),
                ],
                Transform::from_scale(0.1),
                Mesh::load_model("suzanne.obj", &device),
            ),
        ]);

        Ok(Self {
            is_surface_configured: false,
            surface,
            device,
            queue,
            config,

            render_objects,
            light,

            depth_texture,

            camera,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.is_surface_configured = true;

        self.camera.set_aspect(width as f32 / height as f32);

        self.depth_texture = Texture::create_depth_texture(&self.device, &self.config);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
        };

        let view = output.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            for obj in &self.render_objects {
                obj.render(self, &mut render_pass);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())
    }

    pub fn update(&mut self) {
        self.camera.update_uniforms(&self.queue);
    }
}
