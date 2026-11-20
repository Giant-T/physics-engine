use std::f64::consts::PI;

use nalgebra::{Point3, Quaternion, UnitQuaternion, Vector3};
use rand::{RngExt, rngs::ThreadRng};

use super::SceneRuntimeImpl;
use crate::{
    physics::{
        collisions::{Body, DetectionReport, DetectionType},
        tri_mesh::TriMesh,
    },
    renderer::{AABBRenderer, Renderable, Renderer, UIElement},
    transform::Transform,
};

const MESH_PATHS: [&'static str; 12] = [
    "data/armadillo.obj",
    "data/automobile.obj",
    "data/batte.obj",
    "data/bouteille.obj",
    "data/dragon.obj",
    "data/epee.obj",
    "data/hache.obj",
    "data/lapin.obj",
    "data/pieuvre.obj",
    "data/rocher.obj",
    "data/sapin.obj",
    "data/serpent.obj",
];

fn random_rotation(rng: &mut ThreadRng) -> UnitQuaternion<f64> {
    let u1: f64 = rng.random_range(0.0..1.0);
    let u2: f64 = rng.random_range(0.0..1.0);
    let u3: f64 = rng.random_range(0.0..1.0);

    let r1 = (1.0 - u1).sqrt();
    let r2 = u1.sqrt();

    let t1 = 2.0 * PI * u2;
    let t2 = 2.0 * PI * u3;

    let w = r1 * t1.cos();
    let i = r1 * t1.sin();
    let j = r2 * t2.cos();
    let k = r2 * t2.sin();

    UnitQuaternion::from_quaternion(Quaternion::new(w, i, j, k))
}

fn random_direction(rng: &mut ThreadRng) -> Vector3<f64> {
    let z: f64 = rng.random_range(-1.0..1.0);
    let angle: f64 = rng.random_range(0.0..(2.0 * PI));
    let ring: f64 = 0.0_f64.max(1.0 - z * z).sqrt();

    Vector3::new(ring * angle.cos(), ring * angle.sin(), z)
}

pub struct CollisionSceneRuntime {
    bodies: Box<[Body]>,
    half_size: f64,
    bounds_renderer: AABBRenderer,
    detection_type: DetectionType,
    last_detection_report: DetectionReport,
}

impl CollisionSceneRuntime {
    pub fn new(n_bodies: usize, half_size: f64, renderer: &Renderer) -> Self {
        let bodies = Self::build_bodies(n_bodies, half_size, renderer);

        let bounds_renderer = AABBRenderer::from_center_half_extents(
            Vector3::zeros(),
            Vector3::new(half_size, half_size, half_size),
            renderer,
        );

        Self {
            bodies,
            half_size,
            bounds_renderer,
            last_detection_report: DetectionReport::default(),
            detection_type: DetectionType::PairTraversal,
        }
    }

    fn build_bodies(n_bodies: usize, half_size: f64, renderer: &Renderer) -> Box<[Body]> {
        let mut rng = rand::rng();
        let scale = 1.0;
        (0..n_bodies)
            .map(|i| {
                let mesh = TriMesh::load_obj(MESH_PATHS[i]);

                let reach = 0.0_f64.max(half_size - scale);

                let position = Point3::new(
                    rng.random_range(-reach..reach),
                    rng.random_range(-reach..reach),
                    rng.random_range(-reach..reach),
                );

                let rotation = random_rotation(&mut rng);

                let transform = Transform::from_position_and_rotation(position, rotation);

                let velocity = random_direction(&mut rng) * rng.random_range(0.15..0.45);

                let angular_velocity = random_direction(&mut rng) * rng.random_range(0.3..1.2);

                Body::new(mesh, transform, velocity, angular_velocity, renderer)
            })
            .collect()
    }

    fn restrain(body: &mut Body, half_size: f64) {
        let mut position = body.position();
        let mut velocity = body.velocity().clone();

        for (coord, speed) in position.iter_mut().zip(velocity.iter_mut()) {
            if *coord - body.radius() < -half_size && *speed < 0.0 {
                *coord = -half_size + body.radius();
                *speed = -*speed;
            } else if *coord + body.radius() > half_size && *speed > 0.0 {
                *coord = half_size - body.radius();
                *speed = -*speed;
            }
        }

        body.set_position(position);
        body.set_velocity(velocity);
    }

    fn detect(&mut self) {
        self.last_detection_report = self.detection_type.detect(&self.bodies);
    }
}

impl SceneRuntimeImpl for CollisionSceneRuntime {
    fn update(&mut self, h: f64) {
        for body in &mut self.bodies {
            body.step(h);

            Self::restrain(body, self.half_size);
        }

        self.detect();
    }

    fn update_mesh(&mut self, renderer: &Renderer) {
        for body in &mut self.bodies {
            body.update_mesh(renderer);
        }
    }

    fn reset(&mut self, renderer: &Renderer) {
        self.bodies = Self::build_bodies(self.bodies.len(), self.half_size, renderer);
    }

    fn select_from_ray(&mut self, ray: &crate::ray::Ray, renderer: &Renderer) {
        return; // Ne fait rien pour le moment
    }
}

impl UIElement for CollisionSceneRuntime {
    fn ui(&mut self, ui: &mut egui::Ui, renderer: &Renderer) {
        // TODO: ajouter les fonctionnalites
        ui.collapsing("Detections", |ui| {
            self.last_detection_report.ui(ui, renderer);
        });

        ui.label("Afficher les BVHs");
        ui.label("Afficher les collisions");
    }
}

impl Renderable for CollisionSceneRuntime {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        for body in &self.bodies {
            body.render(renderer, render_pass);
        }
        self.bounds_renderer.render(renderer, render_pass);
    }
}
