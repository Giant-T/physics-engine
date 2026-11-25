use std::time::Duration;

use crate::{
    physics::collisions::body::BodyContact,
    renderer::{Renderer, UIElement},
};

#[derive(Default)]
pub struct DetectionReport {
    pub body_pairs: usize,
    pub bounds_tests: usize,
    pub leaf_pairs: usize,
    pub duration: Duration,
    pub tests_count: usize,
    pub contacts: Vec<BodyContact>,
}

impl UIElement for DetectionReport {
    fn ui(&mut self, ui: &mut egui::Ui, _: &Renderer) {
        ui.columns(2, |columns| {
            columns[0].label("Tests AABB:");
            columns[1].label(format!("{}", self.bounds_tests));
        });

        ui.columns(2, |columns| {
            columns[0].label("Couples de feuilles:");
            columns[1].label(format!("{}", self.leaf_pairs));
        });

        ui.columns(2, |columns| {
            columns[0].label("Nombres de tests de contact:");
            columns[1].label(format!("{}", self.tests_count));
        });

        ui.columns(2, |columns| {
            columns[0].label("Contacts:");
            columns[1].label(format!("{}", self.contacts.len()));
        });

        ui.columns(2, |columns| {
            columns[0].label("Temps:");
            columns[1].label(format!("{:?}", self.duration));
        });
    }
}
