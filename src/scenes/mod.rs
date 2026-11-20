use crate::{
    ray::Ray,
    renderer::{Renderable, Renderer, UIElement},
};
use descriptor::SceneDescriptor;
use runtime::SceneRuntime;

pub use runtime::elastic_scene::{Anchoring, ElasticMeshType};
mod descriptor;
pub mod manager;
mod runtime;

pub struct Scene {
    descriptor: &'static SceneDescriptor,
    runtime: SceneRuntime,
    accumulator: f64,
    time: f64,
    stopped: bool,
}

impl Scene {
    pub fn new(descriptor: &'static SceneDescriptor, renderer: &Renderer) -> Self {
        let runtime = SceneRuntime::from_descriptor(descriptor, renderer);
        Self {
            descriptor,
            runtime,
            time: 0.0,
            accumulator: 0.0,
            stopped: true,
        }
    }

    pub fn change_scene(&mut self, descriptor: &'static SceneDescriptor, renderer: &Renderer) {
        self.descriptor = descriptor;
        self.runtime = SceneRuntime::from_descriptor(descriptor, renderer);
        self.accumulator = 0.0;
        self.time = 0.0;
        self.stopped = true;
    }

    fn play(&mut self) {
        self.stopped = false;
    }

    fn pause(&mut self) {
        self.stopped = true;
    }

    fn reset(&mut self, renderer: &Renderer) {
        self.accumulator = 0.0;
        self.time = 0.0;

        self.runtime.reset(renderer);
    }

    pub fn update(&mut self, renderer: &Renderer, delta_time: f64) {
        if self.stopped {
            return;
        }

        self.accumulator += delta_time;

        while self.accumulator >= self.descriptor.time_step {
            self.runtime.update(self.descriptor.time_step);
            self.accumulator -= self.descriptor.time_step;
            self.time += self.descriptor.time_step;
        }

        self.runtime.update_mesh(renderer);
    }

    pub fn select_from_ray(&mut self, ray: &Ray, renderer: &Renderer) {
        self.runtime.select_from_ray(ray, renderer);
    }
}

impl Renderable for Scene {
    fn render(&self, renderer: &crate::renderer::Renderer, render_pass: &mut wgpu::RenderPass) {
        self.runtime.render(renderer, render_pass);
    }
}

impl UIElement for Scene {
    fn ui(&mut self, ui: &mut egui::Ui, renderer: &Renderer) {
        ui.label(self.descriptor.description);
        ui.label(format!("time: {:.2}s", self.time));

        ui.columns(3, |columns| {
            if columns[0].button("Play").clicked() {
                self.play();
            }

            if columns[1].button("Pause").clicked() {
                self.pause();
            }

            if columns[2].button("Reset").clicked() {
                self.reset(renderer);
            }
        });

        self.runtime.ui(ui, renderer);
    }
}
