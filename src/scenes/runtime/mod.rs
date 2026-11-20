use crate::{
    ray::Ray,
    renderer::{Renderable, Renderer, UIElement},
    scenes::runtime::collision_scene::CollisionSceneRuntime,
};

use super::descriptor::{SceneDescriptor, SceneType};
use elastic_scene::ElasticSceneRuntime;

pub mod collision_scene;
pub mod elastic_scene;

trait SceneRuntimeImpl {
    fn update(&mut self, h: f64);

    fn update_mesh(&mut self, renderer: &Renderer);

    fn reset(&mut self, renderer: &Renderer);

    fn select_from_ray(&mut self, ray: &Ray, renderer: &Renderer);
}

pub enum SceneRuntime {
    ElasticScene(ElasticSceneRuntime),
    CollisionScene(CollisionSceneRuntime),
}

impl SceneRuntime {
    pub fn from_descriptor(descriptor: &SceneDescriptor, renderer: &Renderer) -> Self {
        match &descriptor.scene_type {
            SceneType::Elastic {
                mesh_type,
                anchoring,
            } => Self::ElasticScene(ElasticSceneRuntime::new(
                &mesh_type,
                &anchoring,
                descriptor.time_step,
                renderer,
            )),
            SceneType::Collisions {
                n_bodies,
                half_size,
            } => Self::CollisionScene(CollisionSceneRuntime::new(*n_bodies, *half_size, renderer)),
        }
    }

    pub fn update(&mut self, h: f64) {
        match self {
            Self::ElasticScene(elastic_scene) => elastic_scene.update(h),
            Self::CollisionScene(collision_scene) => collision_scene.update(h),
        }
    }

    pub fn update_mesh(&mut self, renderer: &Renderer) {
        match self {
            Self::ElasticScene(elastic_scene) => elastic_scene.update_mesh(renderer),
            Self::CollisionScene(collision_scene) => collision_scene.update_mesh(renderer),
        }
    }

    pub fn reset(&mut self, renderer: &Renderer) {
        match self {
            Self::ElasticScene(elastic_scene) => elastic_scene.reset(renderer),
            Self::CollisionScene(collision_scene) => collision_scene.reset(renderer),
        }
    }

    pub fn select_from_ray(&mut self, ray: &Ray, renderer: &Renderer) {
        match self {
            Self::ElasticScene(elastic_scene) => elastic_scene.select_from_ray(ray, renderer),
            Self::CollisionScene(collision_scene) => collision_scene.select_from_ray(ray, renderer),
        }
    }
}

impl Renderable for SceneRuntime {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        match self {
            Self::ElasticScene(elastic_scene) => elastic_scene.render(renderer, render_pass),
            Self::CollisionScene(collision_scene) => collision_scene.render(renderer, render_pass),
        }
    }
}

impl UIElement for SceneRuntime {
    fn ui(&mut self, ui: &mut egui::Ui, renderer: &Renderer) {
        match self {
            Self::ElasticScene(elastic_scene) => elastic_scene.ui(ui, renderer),
            Self::CollisionScene(collision_scene) => collision_scene.ui(ui, renderer),
        }
    }
}
