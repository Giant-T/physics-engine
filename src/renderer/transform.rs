use cgmath::{
    Deg, EuclideanSpace, InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation3, SquareMatrix,
    Vector3,
    num_traits::{One, Zero},
};

use super::OPENGL_TO_WGPU_MATRIX;

pub struct Transform {
    translation: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: f32,
}

#[allow(dead_code)]
impl Transform {
    pub fn from_position_and_rotation(
        position: Point3<f32>,
        rotation: cgmath::Quaternion<f32>,
    ) -> Self {
        Self {
            translation: position.to_vec(),
            rotation,
            ..Default::default()
        }
    }

    pub fn from_translation(translation: Vector3<f32>) -> Self {
        Self {
            translation: translation,
            ..Default::default()
        }
    }
    pub fn from_rotation(rotation: Quaternion<f32>) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    pub fn from_scale(scale: f32) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    pub fn from_position_pitch_yaw(position: Point3<f32>, pitch: f32, yaw: f32) -> Self {
        Self {
            translation: position.to_vec(),
            rotation: Quaternion::from_angle_y(Rad(yaw)) * Quaternion::from_angle_x(Rad(pitch)),
            ..Default::default()
        }
    }

    pub fn rotate(&mut self, axis: Vector3<f32>, angle: f32) {
        let rotation = Quaternion::from_axis_angle(axis.normalize(), Deg(angle));

        self.rotation = self.rotation * rotation;
    }

    pub fn translate(&mut self, translation: Vector3<f32>) {
        self.translation += translation;
    }

    pub fn scale(&mut self, scaling_factor: f32) {
        self.scale *= scaling_factor;
    }

    pub fn point_to_world(&self, point: Point3<f32>) -> Point3<f32> {
        use cgmath::Transform;

        self.matrix().transform_point(point)
    }

    pub fn point_to_local(&self, point: Point3<f32>) -> Point3<f32> {
        use cgmath::Transform;

        self.matrix().invert().unwrap().transform_point(point)
    }

    pub fn to_buffer(&self) -> [[f32; 4]; 4] {
        (OPENGL_TO_WGPU_MATRIX * self.matrix()).into()
    }

    pub fn position(&self) -> Point3<f32> {
        self.point_to_world(Point3::new(0.0, 0.0, 0.0))
    }

    pub fn forward(&self) -> Vector3<f32> {
        (self.rotation * -Vector3::unit_z()).normalize()
    }

    pub fn up(&self) -> Vector3<f32> {
        (self.rotation * Vector3::unit_y()).normalize()
    }

    pub fn right(&self) -> Vector3<f32> {
        (self.rotation * Vector3::unit_x()).normalize()
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(self.translation);
        let rotation = Matrix4::from(self.rotation);
        let scale = Matrix4::from_scale(self.scale);

        translation * rotation * scale
    }

    pub fn update_rotation_from_pitch_yaw(&mut self, pitch: f32, yaw: f32) {
        self.rotation = Quaternion::from_angle_y(Rad(yaw)) * Quaternion::from_angle_x(Rad(pitch));
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vector3::zero(),
            rotation: Quaternion::one(),
            scale: 1.0,
        }
    }
}
