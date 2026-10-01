use std::time::Duration;

use cgmath::{InnerSpace, Matrix4, SquareMatrix, Vector3, num_traits::Zero};
use wgpu::util::DeviceExt;
use winit::keyboard::KeyCode;

use super::{OPENGL_TO_WGPU_MATRIX, Transform};

pub struct Camera {
    transform: Transform,

    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    view_proj_buffer: wgpu::Buffer,
    position_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(
        device: &wgpu::Device,
        transform: Transform,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let matrix: [[f32; 4]; 4] = (OPENGL_TO_WGPU_MATRIX
            * Self::proj_matrix(aspect, fovy, znear, zfar)
            * transform.matrix().invert().unwrap())
        .into();
        let view_proj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("View Proj Buffer"),
            contents: bytemuck::cast_slice(&[matrix]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let position: [f32; 3] = transform.position().into();
        let position_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Position Buffer"),
            contents: bytemuck::cast_slice(&[position]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("camera_bind_group_layout"),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: view_proj_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: position_buffer.as_entire_binding(),
                },
            ],
            label: Some("camera_bind_group"),
        });

        Self {
            transform,

            aspect,
            fovy,
            znear,
            zfar,

            view_proj_buffer,
            position_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    fn proj_matrix(aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Matrix4<f32> {
        cgmath::perspective(cgmath::Deg(fovy), aspect, znear, zfar)
    }

    fn build_matrix(&self) -> Matrix4<f32> {
        let view = self.transform.matrix().invert().unwrap();

        OPENGL_TO_WGPU_MATRIX
            * Self::proj_matrix(self.aspect, self.fovy, self.znear, self.zfar)
            * view
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue) {
        let matrix: [[f32; 4]; 4] = self.build_matrix().into();
        queue.write_buffer(&self.view_proj_buffer, 0, bytemuck::cast_slice(&[matrix]));
        let position: [f32; 3] = self.transform.position().into();
        queue.write_buffer(&self.position_buffer, 0, bytemuck::cast_slice(&[position]));
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, delta_time: Duration) {
        let delta_time = delta_time.as_secs_f32();
        let transform = camera.transform_mut();

        let forward = transform.forward();
        // let up = transform.up();
        let right = transform.right();

        let mut translation = Vector3::<f32>::zero();

        // Avancer/Reculer
        if self.is_forward_pressed {
            translation += forward;
        }
        if self.is_backward_pressed {
            translation -= forward;
        }

        // Bouger horizontalement
        if self.is_right_pressed {
            translation += right;
        }
        if self.is_left_pressed {
            translation -= right;
        }

        if translation.magnitude2() > 0.0 {
            transform.translate(translation.normalize() * self.speed * delta_time);
        }
    }
}
