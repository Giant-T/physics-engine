use std::time::Instant;

use nalgebra::Vector3;

mod aabb;
mod body;
mod detection_report;
mod narrow_phase;
mod pair_traversal;

use pair_traversal::PairTraversal;

pub use aabb::AABB;
pub use body::Body;
pub use detection_report::DetectionReport;
pub use narrow_phase::ContactDetection;

use crate::physics::collisions::body::BodyContact;

pub enum DetectionType {
    PairTraversal,
    BruteForce,
}

impl DetectionType {
    pub fn detect(&self, bodies: &[Body], contact_detection: &ContactDetection) -> DetectionReport {
        match self {
            DetectionType::PairTraversal => Self::detect_pair_traversal(bodies, contact_detection),
            DetectionType::BruteForce => Self::detect_brute_force(bodies),
        }
    }

    fn test_corners(
        bodies: &[Body],
        vertex_body: usize,
        vertex_triangle: usize,
        triangle_body: usize,
        triangle: usize,
        contacts: &mut Vec<BodyContact>,
        contact_detection: &ContactDetection,
    ) {
        let corners = bodies[triangle_body].mesh().triangles()[triangle];

        let triangle_transform = bodies[triangle_body].transform();
        let triangle_transform = triangle_transform.lock().unwrap();

        let a = triangle_transform.vector_position_to_world(
            &bodies[triangle_body].mesh().vertices()[corners[0] as usize],
        );
        let b = triangle_transform.vector_position_to_world(
            &bodies[triangle_body].mesh().vertices()[corners[1] as usize],
        );
        let c = triangle_transform.vector_position_to_world(
            &bodies[triangle_body].mesh().vertices()[corners[2] as usize],
        );

        let vertex_transform = bodies[vertex_body].transform();
        let vertex_transform = vertex_transform.lock().unwrap();

        let triangle_b = [&a, &b, &c];
        let triangle_a = bodies[vertex_body].mesh().triangles()[vertex_triangle].map(|vertex| {
            vertex_transform
                .vector_position_to_world(&bodies[vertex_body].mesh().vertices()[vertex as usize])
        });

        contacts.extend(
            contact_detection
                .detect(triangle_a, triangle_b, 0.02)
                .into_iter()
                .zip(bodies[vertex_body].mesh().triangles()[vertex_triangle].iter())
                .map(|(contact, vertex)| BodyContact {
                    vertex_body,
                    vertex: *vertex as usize,
                    triangle_body,
                    triangle,
                    contact,
                }),
        );
    }

    fn detect_pair_traversal(
        bodies: &[Body],
        contact_detection: &ContactDetection,
    ) -> DetectionReport {
        let start = Instant::now();
        let count = bodies.len();

        let mut body_pairs = 0;
        let mut leaf_pairs = 0;
        let mut bounds_tests = 0;
        let mut contacts = Vec::new();
        let mut tests_count = 0;

        for a in 0..count {
            for b in (a + 1)..count {
                body_pairs += 1;
                let mut traversal = PairTraversal::new(&bodies[a], &bodies[b]);
                traversal.run();
                bounds_tests += traversal.bounds_tests();
                leaf_pairs += traversal.leaf_pairs().len();

                for pair in traversal.leaf_pairs() {
                    Self::test_corners(
                        bodies,
                        a,
                        pair.a,
                        b,
                        pair.b,
                        &mut contacts,
                        contact_detection,
                    );
                    Self::test_corners(
                        bodies,
                        b,
                        pair.b,
                        a,
                        pair.a,
                        &mut contacts,
                        contact_detection,
                    );
                    tests_count += contact_detection.tests_count() * 2;
                }
            }
        }

        DetectionReport {
            body_pairs,
            leaf_pairs,
            bounds_tests,
            contacts,
            tests_count,
            duration: start.elapsed(),
        }
    }

    fn detect_brute_force(bodies: &[Body]) -> DetectionReport {
        let start = Instant::now();
        let count = bodies.len();

        let mut body_pairs = 0;
        let mut contacts = Vec::new();
        let mut tests_count = 0;

        let world: Vec<Vec<Vector3<f64>>> =
            bodies.iter().map(|body| body.world_vertices()).collect();
        let triangles: Vec<&[[u32; 3]]> =
            bodies.iter().map(|body| body.mesh().triangles()).collect();

        for i in 0..count {
            for j in (i + 1)..count {
                body_pairs += 1;

                if !AABB::overlap(
                    &bodies[i]
                        .bounds()
                        .root_bounds()
                        .transform(&bodies[i].transform().lock().unwrap()),
                    &bodies[j]
                        .bounds()
                        .root_bounds()
                        .transform(&bodies[j].transform().lock().unwrap()),
                ) {
                    continue;
                }

                for vertex_body in [i, j] {
                    let triangle_body = if vertex_body == i { j } else { i };
                    let triangle_body_bounds = bodies[triangle_body]
                        .bounds()
                        .root_bounds()
                        .transform(&bodies[triangle_body].transform().lock().unwrap());
                    let targets = triangles[triangle_body];

                    for vertex in 0..world[vertex_body].len() {
                        let point = &world[vertex_body][vertex];

                        if !triangle_body_bounds.point_is_in(point) {
                            continue;
                        }

                        for t in 0..targets.len() {
                            tests_count += 1;
                            let a = &world[triangle_body][targets[t][0] as usize];
                            let b = &world[triangle_body][targets[t][1] as usize];
                            let c = &world[triangle_body][targets[t][2] as usize];

                            if let Some(contact) =
                                ContactDetection::point_triangle(&point, &a, &b, &c, 0.02)
                            {
                                contacts.push(BodyContact {
                                    vertex_body,
                                    vertex,
                                    triangle_body,
                                    triangle: t,
                                    contact,
                                });
                            }
                        }
                    }
                }
            }
        }

        DetectionReport {
            body_pairs,
            bounds_tests: 0,
            leaf_pairs: 0,
            tests_count,
            contacts,
            duration: start.elapsed(),
        }
    }
}
