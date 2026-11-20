use crate::{
    renderer::{Renderer, UIElement},
    scenes::descriptor::SceneType,
};

use super::{Anchoring, ElasticMeshType, Scene, SceneDescriptor};

const SCENES_DESCRIPTORS: [SceneDescriptor; 4] = [
    SceneDescriptor {
        name: "Test Compression",
        description: "Test de la compression pour le neo-hookeen",
        time_step: 0.05,
        scene_type: SceneType::Elastic {
            mesh_type: ElasticMeshType::Box(4, 4, 4, 0.08),
            anchoring: Anchoring::Bottom(1e-9),
        },
    },
    SceneDescriptor {
        name: "Test poutre",
        description: "Test de la d'une poutre avec des epingles sur sa face gauche",
        time_step: 0.05,
        scene_type: SceneType::Elastic {
            mesh_type: ElasticMeshType::Box(12, 3, 3, 0.08),
            anchoring: Anchoring::LeftFace(1e-9),
        },
    },
    SceneDescriptor {
        name: "Test lapin",
        description: "C'est un joli lapin avec des epingles sous lui",
        time_step: 0.05,
        scene_type: SceneType::Elastic {
            mesh_type: ElasticMeshType::File("data/bunny_coarser.mesh"),
            anchoring: Anchoring::Bottom(0.06),
        },
    },
    SceneDescriptor {
        name: "Test collisions",
        description: "De jolies collisions entre une multitude d'objets differents",
        time_step: 0.05,
        scene_type: SceneType::Collisions {
            n_bodies: 6,
            half_size: 2.5,
        },
    },
];

pub struct SceneManager {
    scene: Scene,
}

impl SceneManager {
    pub fn new(renderer: &Renderer) -> Self {
        let scene = Scene::new(&SCENES_DESCRIPTORS[0], &renderer);
        Self { scene }
    }

    pub fn change_scene(&mut self, descriptor: &'static SceneDescriptor, renderer: &Renderer) {
        self.scene.change_scene(descriptor, renderer);
    }

    pub fn update_scene(&mut self, renderer: &Renderer, delta_time: f64) {
        self.scene.update(renderer, delta_time);
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }
}

impl UIElement for SceneManager {
    fn ui(&mut self, ui: &mut egui::Ui, renderer: &Renderer) {
        egui::Window::new("Simulation").show(ui, |ui| {
            egui::ComboBox::from_label("Scene")
                .selected_text(self.scene.descriptor.name)
                .show_ui(ui, |ui| {
                    for scene_descriptor in &SCENES_DESCRIPTORS {
                        if ui.button(scene_descriptor.name).clicked() {
                            self.change_scene(scene_descriptor, renderer);
                        }
                    }
                });

            self.scene.ui(ui, renderer);
        });
    }
}
