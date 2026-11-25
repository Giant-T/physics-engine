use std::f64::consts::PI;

use nalgebra::{Point3, Quaternion, UnitQuaternion, Vector3};
use rand::{RngExt, SeedableRng, rngs::StdRng};

use super::SceneRuntimeImpl;
use crate::{
    physics::{
        collisions::{Body, ContactDetection, DetectionReport, DetectionType},
        tri_mesh::TriMesh,
    },
    renderer::{AABBRenderer, PointsRenderer, Renderable, Renderer, UIElement, VectorsRenderer},
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

fn random_rotation(rng: &mut StdRng) -> UnitQuaternion<f64> {
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

fn random_direction(rng: &mut StdRng) -> Vector3<f64> {
    let z: f64 = rng.random_range(-1.0..1.0);
    let angle: f64 = rng.random_range(0.0..(2.0 * PI));
    let ring: f64 = 0.0_f64.max(1.0 - z * z).sqrt();

    Vector3::new(ring * angle.cos(), ring * angle.sin(), z)
}

pub struct CollisionDetectionSceneRuntime {
    bodies: Box<[Body]>,
    half_size: f64,
    bounds_renderer: AABBRenderer,
    contacts_renderer: PointsRenderer,
    contact_normals_renderer: VectorsRenderer,
    last_detection_report: DetectionReport,
    last_brute_force_detection_report: DetectionReport,
    show_bounds: bool,
    shown_depth: u32,
    show_contacts: bool,
    compare_with_brute_force: bool,
    contact_detection: ContactDetection,
}

impl CollisionDetectionSceneRuntime {
    pub fn new(n_bodies: usize, half_size: f64, renderer: &Renderer) -> Self {
        let bodies = Self::build_bodies(n_bodies, half_size, renderer);

        let bounds_renderer = AABBRenderer::from_center_half_extents(
            Vector3::zeros(),
            Vector3::new(half_size, half_size, half_size),
            renderer,
        );

        let contacts_renderer = PointsRenderer::from_points(&[], renderer);
        contacts_renderer.set_color(&Vector3::new(1.0, 0.25, 0.0), renderer);

        let contact_normals_renderer = VectorsRenderer::from_vectors_positions(&[], renderer);
        contact_normals_renderer.set_color(&Vector3::new(0.0, 1.0, 0.25), renderer);

        Self {
            bodies,
            half_size,
            bounds_renderer,
            contacts_renderer,
            contact_normals_renderer,
            last_detection_report: DetectionReport::default(),
            last_brute_force_detection_report: DetectionReport::default(),
            compare_with_brute_force: false,
            show_bounds: false,
            shown_depth: 1,
            show_contacts: false,
            contact_detection: ContactDetection::PointTriangle,
        }
    }

    fn build_bodies(n_bodies: usize, half_size: f64, renderer: &Renderer) -> Box<[Body]> {
        let mut rng = StdRng::seed_from_u64(42);
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
        self.last_detection_report =
            DetectionType::PairTraversal.detect(&self.bodies, &self.contact_detection);

        if self.compare_with_brute_force {
            self.last_brute_force_detection_report =
                DetectionType::BruteForce.detect(&self.bodies, &self.contact_detection);
        }
    }
}

impl SceneRuntimeImpl for CollisionDetectionSceneRuntime {
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

        let contact_points: Vec<Vector3<f64>> = self
            .last_detection_report
            .contacts
            .iter()
            .map(|contact| contact.contact.point)
            .collect();
        self.contacts_renderer.set_points(&contact_points, renderer);

        let contact_normals: Vec<(Vector3<f64>, Vector3<f64>)> = self
            .last_detection_report
            .contacts
            .iter()
            .map(|contact| (contact.contact.normal.into_inner(), contact.contact.point))
            .collect();
        self.contact_normals_renderer
            .set_vectors_positions(&contact_normals, renderer);
    }

    fn reset(&mut self, renderer: &Renderer) {
        self.bodies = Self::build_bodies(self.bodies.len(), self.half_size, renderer);
    }

    fn select_from_ray(&mut self, ray: &crate::ray::Ray, renderer: &Renderer) {
        return; // Ne fait rien pour le moment
    }
}

impl UIElement for CollisionDetectionSceneRuntime {
    fn ui(&mut self, ui: &mut egui::Ui, renderer: &Renderer) {
        ui.collapsing("Detections", |ui| {
            egui::ComboBox::from_label("Type de detection de contact")
                .selected_text(format!("{:?}", self.contact_detection))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.contact_detection,
                        ContactDetection::PointTriangle,
                        format!("{:?}", ContactDetection::PointTriangle),
                    );
                    ui.selectable_value(
                        &mut self.contact_detection,
                        ContactDetection::PointEdge,
                        format!("{:?}", ContactDetection::PointEdge),
                    );
                    ui.selectable_value(
                        &mut self.contact_detection,
                        ContactDetection::EdgeEdge,
                        format!("{:?}", ContactDetection::EdgeEdge),
                    );
                });

            ui.label("Pair Traversal");
            self.last_detection_report.ui(ui, renderer);

            ui.checkbox(
                &mut self.compare_with_brute_force,
                "Comparer avec la force brute",
            );

            if self.compare_with_brute_force {
                ui.separator();
                ui.label("Force Brute");
                self.last_brute_force_detection_report.ui(ui, renderer);
            }
        });

        let response = ui.checkbox(&mut self.show_bounds, "Afficher les BVHs");
        if response.changed() {
            for body in &mut self.bodies {
                body.show_bounds = self.show_bounds;
            }
        }
        if self.show_bounds {
            let max_depth = self
                .bodies
                .iter()
                .map(|body| body.bounds().max_depth())
                .max()
                .unwrap();
            let response = ui.add(
                egui::Slider::new(&mut self.shown_depth, 1..=max_depth)
                    .show_value(true)
                    .text("Profondeur"),
            );
            if response.changed() {
                for body in &mut self.bodies {
                    body.bounds_mut().set_shown_depth(self.shown_depth);
                }
            }
        }

        ui.checkbox(
            &mut self.show_contacts,
            "Afficher les contacts (points et normales)",
        );
    }
}

impl Renderable for CollisionDetectionSceneRuntime {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        for body in &self.bodies {
            body.render(renderer, render_pass);
        }

        self.bounds_renderer.render(renderer, render_pass);

        if self.show_contacts {
            self.contacts_renderer.render(renderer, render_pass);
            self.contact_normals_renderer.render(renderer, render_pass);
        }
    }
}
