use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use nalgebra::{Point3, UnitQuaternion, Vector3};
use winit::{event::WindowEvent, window::Window};

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

pub struct Renderer {
    is_surface_configured: bool,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pixels_per_point: f32,

    ui_renderer: egui_wgpu::Renderer,
    ui_ctx: egui::Context,
    ui_state: egui_winit::State,

    render_objects: Box<[RenderObject]>,
    light: PointLight,

    depth_texture: Texture,

    camera: Camera,
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
            Point3::new(0.0, 0.0, 5.0),
            0.0,
            0.0,
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
                "Stanford Bunny",
                &[
                    Some(camera.bind_group_layout()),
                    Some(light.bind_group_layout()),
                ],
                Transform::from_rotation(UnitQuaternion::from_axis_angle(
                    &Vector3::y_axis(),
                    90.0f32.to_radians(),
                )),
                Mesh::load_model("stanford-bunny.obj", &device),
            ),
            RenderObject::new(
                &device,
                &config,
                "Suzanne",
                &[
                    Some(camera.bind_group_layout()),
                    Some(light.bind_group_layout()),
                ],
                Transform::from_scale(0.1),
                Mesh::load_model("suzanne.obj", &device),
            ),
        ]);

        let ui_renderer = egui_wgpu::Renderer::new(
            &device,
            surface_format,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: true,
                predictable_texture_filtering: false,
            },
        );
        let ui_ctx = egui::Context::default();
        let ui_state = egui_winit::State::new(
            ui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );

        Ok(Self {
            is_surface_configured: false,
            surface,
            device,
            queue,
            config,
            pixels_per_point: window.scale_factor() as f32,

            ui_renderer,
            ui_ctx,
            ui_state,

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

    pub fn render(&mut self, window: &Window, delta_time: f32) -> anyhow::Result<()> {
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

        self.render_scene(&view, &mut encoder);
        self.render_ui(&view, &mut encoder, window, delta_time);

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())
    }

    fn render_scene(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
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

    fn render_ui(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        delta_time: f32,
    ) {
        let input = self.ui_state.take_egui_input(window);
        let fps = 1.0 / delta_time;
        let mut full_output = self.ui_ctx.run_ui(input, |ui| {
            ui.label(format!("{delta_time:.6} delta time"));
            ui.label(format!("{fps:.0} fps"));

            egui::Window::new("Objets").show(ui, |ui| {
                for obj in &mut self.render_objects {
                    obj.ui(ui);
                }
            });
        });

        let clipped_primitives = self
            .ui_ctx
            .tessellate(full_output.shapes, self.pixels_per_point);

        for (id, image_deltas) in &full_output.textures_delta.set {
            for image_delta in image_deltas {
                self.ui_renderer
                    .update_texture(&self.device, &self.queue, *id, image_delta);
            }
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.pixels_per_point,
        };

        self.ui_renderer.update_buffers(
            &self.device,
            &self.queue,
            encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        {
            let mut render_pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("UI Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                })
                .forget_lifetime();

            self.ui_renderer
                .render(&mut render_pass, &clipped_primitives, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            self.ui_renderer.free_texture(id);
        }

        full_output.textures_delta.clear();
    }

    pub fn ui_window_event(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.ui_state.on_window_event(window, event);
    }

    pub fn update(&mut self) {
        self.camera.update_uniforms(&self.queue);
        for obj in &self.render_objects {
            obj.update_uniforms(&self.queue);
        }
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}
