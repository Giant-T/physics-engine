use std::ops::Mul;

use nalgebra::{Matrix3, Matrix4, Point3, RealField, Unit, UnitQuaternion, Vector3, Vector4};
use num_traits::ToPrimitive;

use crate::renderer::{Renderer, UIElement};

pub struct Transform<T: RealField> {
    translation: Vector3<T>,
    rotation: UnitQuaternion<T>,
    scale: T,
}

impl<T: RealField + ToPrimitive + Copy + Mul<Matrix3<T>, Output = Matrix3<T>>> Transform<T> {
    pub fn from_position_and_rotation(position: Point3<T>, rotation: UnitQuaternion<T>) -> Self {
        Self {
            translation: position.coords,
            rotation,
            ..Default::default()
        }
    }

    pub fn from_translation(translation: Vector3<T>) -> Self {
        Self {
            translation: translation,
            ..Default::default()
        }
    }

    pub fn from_rotation(rotation: UnitQuaternion<T>) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    pub fn from_scale(scale: T) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    pub fn from_position_pitch_yaw(position: Point3<T>, pitch: T, yaw: T) -> Self {
        Self {
            translation: position.coords,
            rotation: UnitQuaternion::from_axis_angle(&Vector3::y_axis(), yaw)
                * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), pitch),
            ..Default::default()
        }
    }

    pub fn rotate(&mut self, axis: Vector3<T>, angle: T) {
        let rotation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(axis), angle);

        self.rotation = &self.rotation * rotation;
    }

    pub fn translate(&mut self, translation: Vector3<T>) {
        self.translation += translation;
    }

    pub fn set_translation(&mut self, translation: Vector3<T>) {
        self.translation = translation;
    }

    pub fn scale(&mut self, scaling_factor: T) {
        self.scale *= scaling_factor;
    }

    pub fn point_to_world(&self, point: &Point3<T>) -> Point3<T> {
        self.matrix().transform_point(point)
    }

    pub fn vector_position_to_world(&self, point: &Vector3<T>) -> Vector3<T> {
        (self.matrix() * Vector4::new(point.x, point.y, point.z, T::one())).xyz()
    }

    pub fn vector_to_world(&self, vector: &Vector3<T>) -> Vector3<T> {
        (self.scale * *self.rotation.to_rotation_matrix().matrix()).abs() * vector
    }

    pub fn point_to_local(&self, point: &Point3<T>) -> Point3<T> {
        self.matrix().try_inverse().unwrap().transform_point(point)
    }

    pub fn vector_to_local(&self, vector: &Vector3<T>) -> Vector3<T> {
        self.matrix()
            .try_inverse()
            .unwrap()
            .transform_vector(vector)
    }

    pub fn to_buffer(&self) -> [[f32; 4]; 4] {
        self.matrix().map(|x| x.to_f32().unwrap()).into()
    }

    pub fn position(&self) -> Point3<T> {
        self.point_to_world(&Point3::origin())
    }

    pub fn position_buffer(&self) -> [f32; 3] {
        self.position().map(|x| x.to_f32().unwrap()).into()
    }

    pub fn forward(&self) -> Vector3<T> {
        (&self.rotation * -Vector3::z()).normalize()
    }

    pub fn up(&self) -> Vector3<T> {
        (&self.rotation * Vector3::y()).normalize()
    }

    pub fn up_buffer(&self) -> [f32; 3] {
        self.up().map(|x| x.to_f32().unwrap()).into()
    }

    pub fn right(&self) -> Vector3<T> {
        (&self.rotation * Vector3::x()).normalize()
    }

    pub fn right_buffer(&self) -> [f32; 3] {
        self.right().map(|x| x.to_f32().unwrap()).into()
    }

    pub fn matrix(&self) -> Matrix4<T> {
        let translation = Matrix4::new_translation(&self.translation);
        let rotation = Matrix4::from(self.rotation.clone());
        let scale = Matrix4::new_scaling(self.scale.clone());

        translation * rotation * scale
    }

    pub fn update_rotation_from_pitch_yaw(&mut self, pitch: T, yaw: T) {
        self.rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), yaw)
            * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), pitch);
    }
}

impl<T: RealField + egui::emath::Numeric> UIElement for Transform<T> {
    fn ui(&mut self, ui: &mut egui::Ui, _: &Renderer) {
        ui.columns(2, |columns| {
            columns[0].label("Position:");
            columns[1].columns(3, |columns| {
                columns[0].add(egui::DragValue::new(&mut self.translation.x).speed(0.01));
                columns[1].add(egui::DragValue::new(&mut self.translation.y).speed(0.01));
                columns[2].add(egui::DragValue::new(&mut self.translation.z).speed(0.01));
            });
        });
        ui.columns(2, |columns| {
            columns[0].label("Scale:");
            columns[1].add(
                egui::DragValue::new(&mut self.scale)
                    .range(0.0..=100.0)
                    .speed(0.1),
            )
        });
    }
}

impl<T: RealField> Default for Transform<T> {
    fn default() -> Self {
        Self {
            translation: Vector3::zeros(),
            rotation: UnitQuaternion::default(),
            scale: T::one(),
        }
    }
}
