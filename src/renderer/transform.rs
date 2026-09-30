use cgmath::{Deg, Matrix, Matrix4, Point3, SquareMatrix, Vector3};

pub struct Transform {
    matrix: Matrix4<f32>,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            matrix: Matrix4::identity(),
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

    pub fn convert_point(&self, point: Point3<f32>, destination: Self) -> Point3<f32> {
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
}
