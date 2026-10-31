use crate::renderer::{Renderable, Renderer, UIElement};
use object::SceneObject;

mod object;

pub struct SceneDescriptor {
    pub name: &'static str,
    pub mesh_path: &'static str,
}

pub struct Scene {
    descriptor: SceneDescriptor,
    object: SceneObject,
}

impl Scene {
    pub fn build_scene(descriptor: SceneDescriptor, renderer: &Renderer) -> Self {
        let object = SceneObject::new(descriptor.mesh_path, renderer);

        Self { descriptor, object }
    }

    pub fn update() {
        todo!();
    }
}

impl Renderable for Scene {
    fn render(&self, renderer: &crate::renderer::Renderer, render_pass: &mut wgpu::RenderPass) {
        self.object.render(renderer, render_pass);
    }
}

impl UIElement for Scene {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Scene: {}", self.descriptor.name));
    }
}
