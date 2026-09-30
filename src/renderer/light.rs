use cgmath::Vector3;

use super::Transform;

pub struct PointLight {
    position: Vector3<f32>,
    transform: Transform,
}

impl PointLight {
    pub fn new(position: Vector3<f32>, transform: Transform) -> Self {
        Self {
            position,
            transform,
        }
    }
}
