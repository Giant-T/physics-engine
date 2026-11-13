use std::{rc::Rc, sync::Mutex};

use nalgebra::{Point3, Vector3};

use crate::{
    physics::{
        elastic_model::{ElasticModel, LameParameters, model_type::ElasticModelType},
        solver::{ImplicitSolver, SolverOptions},
        tet_mesh::TetMesh,
    },
    ray::{Ray, RayDistance},
    renderer::{MeshRenderer, PointsRenderer, Renderable, Renderer, UIElement},
};

const POINT_SELECTION_MIN_DIST: f64 = 0.1;

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
    selected_point: Option<Vector3<f64>>,
    mesh_renderer: MeshRenderer,
    pins_renderer: PointsRenderer,
    selected_point_renderer: PointsRenderer,
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
        let pins_renderer = PointsRenderer::from_points(&tet_mesh.pins(), renderer);
        let selected_point_renderer = PointsRenderer::from_points(&[], renderer);

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

        {
            let model = model.lock().unwrap();
            log::info!(
                "Energie potentielle initiale: {}",
                model.potential_energy(&solver.x)
            );
        }

        Self {
            name: mesh_type.name(),
            model,
            solver,
            selected_point: None,
            mesh_renderer,
            pins_renderer,
            selected_point_renderer,
        }
    }

    pub fn update(&mut self) {
        self.solver.step();
    }

    pub fn reset(&mut self) {
        self.solver.reset();
        self.selected_point = None;
    }

    pub fn update_mesh(&mut self, renderer: &Renderer) {
        self.mesh_renderer
            .update_mesh(&renderer, &self.solver.current_positions());
    }

    pub fn model_mut(&mut self) -> &mut Rc<Mutex<ElasticModel>> {
        &mut self.model
    }

    pub fn select_from_ray(&mut self, ray: &Ray, renderer: &Renderer) {
        let model = self.model.lock().unwrap();
        let mut closest = None;
        let mut closest_dist: Option<RayDistance> = None;

        for idx in 0..model.mesh().node_count() {
            let vector = Vector3::from(self.solver.x.fixed_rows::<3>(3 * idx));
            if let Some(dist) = ray.dist_with_point(&Point3::from(vector)) {
                if dist.distance > POINT_SELECTION_MIN_DIST {
                    continue;
                }

                log::info!("dist: {dist:?}");

                if closest_dist.is_none() || dist.distance < closest_dist.clone().unwrap().distance
                {
                    closest_dist = Some(dist);
                    closest = Some(vector);
                }
            }
        }

        self.selected_point = closest;
        if let Some(point) = closest {
            self.selected_point_renderer.set_points(&[point], renderer);
        } else {
            self.selected_point_renderer.set_points(&[], renderer);
        }
    }
}

impl UIElement for SceneObject {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.model.lock().unwrap().ui(ui);
        self.solver.ui(ui);
    }
}

impl Renderable for SceneObject {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        self.mesh_renderer.render(renderer, render_pass);
        self.pins_renderer.render(renderer, render_pass);

        if self.selected_point.is_some() {
            self.selected_point_renderer.render(renderer, render_pass);
        }
    }
}
