use crate::renderer::{Renderable, Renderer, UIElement};
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
        }
    }

    fn reset(&mut self) {
        self.accumulator = 0.0;
        self.time = 0.0;

        self.object.reset();
    }

    pub fn update(&mut self, renderer: &Renderer, delta_time: f64) {
        self.accumulator += delta_time;

        while self.accumulator >= self.descriptor.time_step {
            self.object.update();
            self.accumulator -= self.descriptor.time_step;
            self.time += self.descriptor.time_step;
        }

        self.object.update_mesh(renderer);
    }
}

impl Renderable for Scene {
    fn render(&self, renderer: &crate::renderer::Renderer, render_pass: &mut wgpu::RenderPass) {
        self.object.render(renderer, render_pass);
    }
}

impl UIElement for Scene {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Window::new(format!("Scene: {}", self.descriptor.name)).show(ui, |ui| {
            ui.label(self.descriptor.description);
            ui.label(format!("time: {:.2}s", self.time));
            if ui.button("Reset").clicked() {
                self.reset();
            }
            self.object.ui(ui);
        });
    }
}
