use crate::{
    ray::Ray,
    renderer::{Renderable, Renderer, UIElement},
};
use object::SceneObject;

pub use object::{Anchoring, ElasticMeshType};
mod object;

pub struct SceneDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub mesh_type: ElasticMeshType,
    pub time_step: f64,
    pub anchoring: Anchoring,
}

pub struct Scene {
    descriptor: SceneDescriptor,
    object: SceneObject,
    accumulator: f64,
    time: f64,
    stopped: bool,
}

impl Scene {
    pub fn build_scene(descriptor: SceneDescriptor, renderer: &Renderer) -> Self {
        let object = SceneObject::new(
            &descriptor.mesh_type,
            &descriptor.anchoring,
            descriptor.time_step,
            renderer,
        );

        Self {
            descriptor,
            object,
            time: 0.0,
            accumulator: 0.0,
            stopped: true,
        }
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

        self.object.reset(renderer);
    }

    pub fn update(&mut self, renderer: &Renderer, delta_time: f64) {
        if self.stopped {
            return;
        }

        self.accumulator += delta_time;

        while self.accumulator >= self.descriptor.time_step {
            self.object.update();
            self.accumulator -= self.descriptor.time_step;
            self.time += self.descriptor.time_step;
        }

        self.object.update_mesh(renderer);
    }

    pub fn select_from_ray(&mut self, ray: &Ray, renderer: &Renderer) {
        self.object.select_from_ray(ray, renderer);
    }
}

impl Renderable for Scene {
    fn render(&self, renderer: &crate::renderer::Renderer, render_pass: &mut wgpu::RenderPass) {
        self.object.render(renderer, render_pass);
    }
}

impl UIElement for Scene {
    fn ui(&mut self, ui: &mut egui::Ui, renderer: &Renderer) {
        egui::Window::new(format!("Scene: {}", self.descriptor.name)).show(ui, |ui| {
            ui.label(self.descriptor.description);
            ui.label(format!("time: {:.2}s", self.time));

            ui.columns(3, |columns| {
                if columns[0].button("play").clicked() {
                    self.play();
                }

                if columns[1].button("pause").clicked() {
                    self.pause();
                }

                if columns[2].button("Reset").clicked() {
                    self.reset(renderer);
                }
            });

            self.object.ui(ui, renderer);
        });
    }
}
