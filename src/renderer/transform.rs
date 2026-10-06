use nalgebra::{Matrix4, Point3, Unit, UnitQuaternion, Vector3};

pub struct Transform {
    translation: Vector3<f32>,
    rotation: UnitQuaternion<f32>,
    scale: f32,
}

#[allow(dead_code)]
impl Transform {
    pub fn from_position_and_rotation(
        position: Point3<f32>,
        rotation: UnitQuaternion<f32>,
    ) -> Self {
        Self {
            translation: position.coords,
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
    pub fn from_rotation(rotation: UnitQuaternion<f32>) -> Self {
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
            translation: position.coords,
            rotation: UnitQuaternion::from_axis_angle(&Vector3::y_axis(), yaw)
                * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), pitch),
            ..Default::default()
        }
    }

    pub fn rotate(&mut self, axis: Vector3<f32>, angle: f32) {
        let rotation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(axis), angle);

        self.rotation = self.rotation * rotation;
    }

    pub fn translate(&mut self, translation: Vector3<f32>) {
        self.translation += translation;
    }

    pub fn scale(&mut self, scaling_factor: f32) {
        self.scale *= scaling_factor;
    }

    pub fn point_to_world(&self, point: Point3<f32>) -> Point3<f32> {
        self.matrix().transform_point(&point)
    }

    pub fn point_to_local(&self, point: Point3<f32>) -> Point3<f32> {
        self.matrix().try_inverse().unwrap().transform_point(&point)
    }

    pub fn to_buffer(&self) -> [[f32; 4]; 4] {
        self.matrix().into()
    }

    pub fn position(&self) -> Point3<f32> {
        self.point_to_world(Point3::new(0.0, 0.0, 0.0))
    }

    pub fn forward(&self) -> Vector3<f32> {
        (self.rotation * -Vector3::z()).normalize()
    }

    pub fn up(&self) -> Vector3<f32> {
        (self.rotation * Vector3::y()).normalize()
    }

    pub fn right(&self) -> Vector3<f32> {
        (self.rotation * Vector3::x()).normalize()
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::new_translation(&self.translation);
        let rotation = Matrix4::from(self.rotation);
        let scale = Matrix4::new_scaling(self.scale);

        translation * rotation * scale
    }

    pub fn update_rotation_from_pitch_yaw(&mut self, pitch: f32, yaw: f32) {
        self.rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), yaw)
            * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), pitch);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
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

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vector3::zeros(),
            rotation: UnitQuaternion::default(),
            scale: 1.0,
        }
    }
}
