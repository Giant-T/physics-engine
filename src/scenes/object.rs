use std::{rc::Rc, sync::Mutex};

use crate::{
    physics::{
        elastic_model::{ElasticModel, LameParameters},
        solver::{ImplicitSolver, SolverOptions},
        tet_mesh::TetMesh,
    },
    renderer::{MeshRenderer, Renderable, Renderer, UIElement},
};

pub struct SceneObject {
    name: &'static str,
    model: Rc<Mutex<ElasticModel>>,
    solver: ImplicitSolver,
    mesh_renderer: MeshRenderer,
}

impl SceneObject {
    pub fn new(mesh_path: &'static str, time_step: f64, renderer: &Renderer) -> Self {
        let tet_mesh = TetMesh::load_mesh_file(mesh_path);
        let mesh_renderer = MeshRenderer::from_tet_mesh(&tet_mesh, renderer);

        let model = Rc::new(Mutex::new(ElasticModel::new(
            tet_mesh,
            LameParameters::from_young_poisson(3.0e6, 0.4),
            1000.0,
        )));

        let solver = ImplicitSolver::new(
            model.clone(),
            SolverOptions {
                time_step,
                ..Default::default()
            },
        );

        Self {
            name: mesh_path.split('/').last().unwrap(),
            model,
            solver,
            mesh_renderer,
        }
    }

    pub fn update(&mut self) {
        self.solver.step();
    }

    pub fn update_mesh(&mut self, renderer: &Renderer) {
        self.mesh_renderer
            .update_mesh(&renderer, &self.solver.current_positions());
    }
}

impl UIElement for SceneObject {
    fn ui(&mut self, ui: &mut egui::Ui) {}
}

impl Renderable for SceneObject {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        self.mesh_renderer.render(renderer, render_pass);
    }
}
