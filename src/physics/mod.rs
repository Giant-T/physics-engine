use nalgebra::Vector3;

pub mod elastic_model;
mod linalg;
pub mod solver;
pub mod tet_mesh;

pub const GRAVITY: Vector3<f64> = Vector3::new(0.0, -9.81, 0.0);
