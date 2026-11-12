use std::{rc::Rc, sync::Mutex};

use crate::{
    physics::{
        elastic_model::{ElasticModel, LameParameters, model_type::ElasticModelType},
        solver::{ImplicitSolver, SolverOptions},
        tet_mesh::TetMesh,
    },
    renderer::{MeshRenderer, PinsRenderer, Renderable, Renderer, UIElement},
};

pub enum Anchoring {
    LeftFace(f64),
    Top(f64),
    Bottom(f64),
}

pub enum ElasticMeshType {
    Box(usize, usize, usize, f64),
    File(&'static str),
}

impl ElasticMeshType {
    pub fn name(&self) -> String {
        match self {
            Self::Box(cells_x, cells_y, cells_z, cell_size) => {
                format!("Boite {cells_x}X{cells_y}X{cells_z} cell_size: {cell_size}")
            }
            Self::File(mesh_path) => mesh_path.split('/').last().unwrap().to_owned(),
        }
    }
}

pub struct SceneObject {
    name: String,
    model: Rc<Mutex<ElasticModel>>,
    solver: ImplicitSolver,
    mesh_renderer: MeshRenderer,
    pins_renderer: PinsRenderer,
}

impl SceneObject {
    pub fn new(
        mesh_type: &ElasticMeshType,
        anchoring: &Anchoring,
        time_step: f64,
        renderer: &Renderer,
    ) -> Self {
        let mut tet_mesh = match mesh_type {
            ElasticMeshType::File(mesh_path) => TetMesh::load_mesh_file(mesh_path),
            ElasticMeshType::Box(cells_x, cells_y, cells_z, cell_size) => {
                TetMesh::box_mesh(*cells_x, *cells_y, *cells_z, *cell_size)
            }
        };

        match anchoring {
            Anchoring::LeftFace(tolerance) => {
                tet_mesh.pin_left_face(*tolerance);
            }
            Anchoring::Top(height_fraction) => {
                tet_mesh.pin_above(*height_fraction);
            }
            Anchoring::Bottom(height_fraction) => {
                tet_mesh.pin_below(*height_fraction);
            }
        }

        let mesh_renderer = MeshRenderer::from_tet_mesh(&tet_mesh, renderer);
        let pins_renderer = PinsRenderer::from_pins(&tet_mesh.pins(), renderer);

        let model = Rc::new(Mutex::new(ElasticModel::new(
            tet_mesh,
            LameParameters::from_young_poisson(3.0e6, 0.4),
            1000.0,
            ElasticModelType::StVK,
        )));

        let solver = ImplicitSolver::new(
            model.clone(),
            SolverOptions {
                time_step,
                ..Default::default()
            },
        );

        Self {
            name: mesh_type.name(),
            model,
            solver,
            mesh_renderer,
            pins_renderer,
        }
    }

    pub fn update(&mut self) {
        self.solver.step();
    }

    pub fn reset(&mut self) {
        self.solver.reset();
    }

    pub fn update_mesh(&mut self, renderer: &Renderer) {
        self.mesh_renderer
            .update_mesh(&renderer, &self.solver.current_positions());
    }

    pub fn model_mut(&mut self) -> &mut Rc<Mutex<ElasticModel>> {
        &mut self.model
    }
}

impl UIElement for SceneObject {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.model.lock().unwrap().ui(ui);
    }
}

impl Renderable for SceneObject {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        self.mesh_renderer.render(renderer, render_pass);
        self.pins_renderer.render(renderer, render_pass);
    }
}
