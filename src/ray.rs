use nalgebra::{Point3, Vector3, distance};

#[derive(Debug)]
pub struct Ray {
    origin: Point3<f64>,
    direction: Vector3<f64>,
}

#[derive(Clone, Debug)]
pub struct RayDistance {
    pub t: f64,
    pub distance: f64,
}

impl Ray {
    pub fn new(origin: Point3<f64>, direction: Vector3<f64>) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn value_at(&self, t: f64) -> Option<Point3<f64>> {
        if t < 0.0 {
            return None;
        }
        Some(self.origin + self.direction * t)
    }

    pub fn dist_with_point(&self, point: &Point3<f64>) -> Option<RayDistance> {
        let t = (point - self.origin).dot(&self.direction);

        let point_on_ray = self.value_at(t)?;

        return Some(RayDistance {
            t,
            distance: distance(&point, &point_on_ray),
        });
    }

    pub fn origin(&self) -> &Point3<f64> {
        &self.origin
    }

    pub fn direction(&self) -> &Vector3<f64> {
        &self.direction
    }
}
