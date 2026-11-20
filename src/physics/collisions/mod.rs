use std::time::{Duration, Instant};

mod aabb;
mod body;
mod pair_traversal;

use pair_traversal::PairTraversal;

pub use aabb::AABB;
pub use body::Body;

use crate::renderer::UIElement;

pub struct DetectionReport {
    pub body_pairs: usize,
    pub bounds_tests: usize,
    pub leaf_pairs: usize,
    pub duration: Duration,
}

impl Default for DetectionReport {
    fn default() -> Self {
        Self {
            body_pairs: Default::default(),
            bounds_tests: Default::default(),
            leaf_pairs: Default::default(),
            duration: Default::default(),
        }
    }
}

impl UIElement for DetectionReport {
    fn ui(&mut self, ui: &mut egui::Ui, _: &crate::renderer::Renderer) {
        ui.columns(2, |columns| {
            columns[0].label("Tests AABB");
            columns[1].label(format!("{}", self.bounds_tests));
        });

        ui.columns(2, |columns| {
            columns[0].label("Couples de feuilles");
            columns[1].label(format!("{}", self.leaf_pairs));
        });

        ui.columns(2, |columns| {
            columns[0].label("Temps");
            columns[1].label(format!("{:?}", self.duration));
        });
    }
}

pub enum DetectionType {
    PairTraversal,
    // BruteForce,
}

impl DetectionType {
    pub fn detect(&self, bodies: &[Body]) -> DetectionReport {
        let start = Instant::now();
        let count = bodies.len();

        let mut body_pairs = 0;
        let mut leaf_pairs = 0;
        let mut bounds_tests = 0;

        for a in 0..count {
            for b in (a + 1)..count {
                body_pairs += 1;
                let mut traversal = PairTraversal::new(&bodies[a], &bodies[b]);
                traversal.run();
                bounds_tests += traversal.bounds_tests();
                leaf_pairs += traversal.leaf_pairs().len();

                // for pair in traversal.leaf_pairs() { TODO
                // }
            }
        }

        DetectionReport {
            body_pairs,
            leaf_pairs,
            bounds_tests,
            duration: start.elapsed(),
        }
    }
}

struct TrianglePair {
    a: usize,
    b: usize,
}
