use cgmath::{Deg, Matrix, Matrix4, Point3, SquareMatrix, Vector3};

use crate::renderer::OPENGL_TO_WGPU_MATRIX;

pub struct Transform {
    matrix: Matrix4<f32>,
}

#[allow(dead_code)]
impl Transform {
    pub fn identity() -> Self {
        Self {
            matrix: Matrix4::identity(),
        }
    }

    pub fn from_rotation(axis: Vector3<f32>, angle: f32) -> Self {
        Self {
            matrix: Matrix4::identity() * Matrix4::from_axis_angle(axis, Deg(angle)),
        }
    }

    pub fn from_scale(scaling_factor: f32) -> Self {
        Self {
            matrix: Matrix4::identity() * Matrix4::from_scale(scaling_factor),
        }
    }

    pub fn rotate(&mut self, axis: Vector3<f32>, angle: f32) {
        self.matrix = self.matrix * Matrix4::from_axis_angle(axis, Deg(angle));
    }

    pub fn translate(&mut self, translation: Vector3<f32>) {
        self.matrix = self.matrix * Matrix4::from_translation(translation);
    }

    pub fn scale(&mut self, scaling_factor: f32) {
        self.matrix = self.matrix * Matrix4::from_scale(scaling_factor);
    }

    pub fn convert_point_to(&self, point: Point3<f32>, destination: Self) -> Point3<f32> {
        use cgmath::Transform;
        let Some(inverse_dest) = destination.matrix.invert() else {
            return point;
        };

        return (inverse_dest * self.matrix)
            .invert()
            .unwrap()
            .transpose()
            .transform_point(point);
    }

    pub fn to_uniform(&self) -> [[f32; 4]; 4] {
        (OPENGL_TO_WGPU_MATRIX * self.matrix).into()
    }
}
