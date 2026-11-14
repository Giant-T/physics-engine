use std::{f64::consts::PI, ops::AddAssign, rc::Rc, sync::Mutex};

use egui::Color32;
use nalgebra::{Point3, Rotation3, Vector3};

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
    selected_point: Option<usize>,
    mesh_renderer: MeshRenderer,
    pins_renderer: PointsRenderer,
    selected_point_renderer: PointsRenderer,
    max_y: f64,
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

        let pins_renderer = PointsRenderer::from_points(&solver.pins(), renderer);
        let selected_point_renderer = PointsRenderer::from_points(&[], renderer);

        let max_y = solver
            .x
            .as_slice()
            .chunks_exact(3)
            .fold(solver.x[1], |acc, v| acc.max(v[1]))
            + 0.2;

        Self {
            name: mesh_type.name(),
            model,
            solver,
            selected_point: None,
            mesh_renderer,
            pins_renderer,
            selected_point_renderer,
            max_y,
        }
    }

    pub fn update(&mut self) {
        self.restrain();
        self.solver.step();
    }

    pub fn reset(&mut self, renderer: &Renderer) {
        self.solver.reset();
        self.selected_point = None;
        self.pins_renderer.set_points(&self.solver.pins(), renderer);
        self.max_y = self
            .solver
            .x
            .as_slice()
            .chunks_exact(3)
            .fold(self.solver.x[1], |acc, v| acc.max(v[1]))
            + 0.2;
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

                if closest_dist.is_none() || dist.distance < closest_dist.clone().unwrap().distance
                {
                    closest_dist = Some(dist);
                    closest = Some(idx);
                }
            }
        }

        self.selected_point = closest;
        if let Some(idx) = closest {
            let vector = Vector3::from(self.solver.x.fixed_rows::<3>(3 * idx));
            self.selected_point_renderer.set_points(&[vector], renderer);
        } else {
            self.selected_point_renderer.set_points(&[], renderer);
        }
    }

    fn pin_selected_point(&mut self, renderer: &Renderer) {
        let Some(idx) = self.selected_point else {
            return;
        };

        self.model.lock().unwrap().mesh_mut().pin(idx);
        self.pins_renderer.set_points(&self.solver.pins(), renderer);
    }

    fn impulse(&mut self) {
        let model = self.model.lock().unwrap();
        let impulse = Vector3::new(1.5, 0.0, 0.4);
        for idx in 0..model.mesh().node_count() {
            if !model.mesh().is_pinned(idx) {
                self.solver
                    .v
                    .fixed_rows_mut::<3>(3 * idx)
                    .add_assign(&impulse);
            }
        }
    }

    fn rotate_around_y(&mut self, renderer: &Renderer) {
        let rotation = Rotation3::from_axis_angle(&Vector3::y_axis(), PI / 2.0);
        let model = self.model.lock().unwrap();

        for idx in 0..model.mesh().node_count() {
            let position: Vector3<f64> = self.solver.x.fixed_rows::<3>(idx * 3).into();
            self.solver
                .x
                .fixed_rows_mut::<3>(idx * 3)
                .copy_from(&rotation.transform_vector(&position));
        }
        drop(model);

        self.pins_renderer.set_points(&self.solver.pins(), renderer);

        self.solver.reset_v();
    }

    fn restrain(&mut self) {
        let model = self.model.lock().unwrap();

        for idx in (0..model.mesh().node_count()).step_by(3).skip(1) {
            self.solver.x[idx] = self.solver.x[idx].min(self.max_y);
        }
    }
}

impl UIElement for SceneObject {
    fn ui(&mut self, ui: &mut egui::Ui, renderer: &Renderer) {
        ui.columns(2, |columns| {
            if columns[0].button("Rotate").clicked() {
                self.rotate_around_y(renderer);
            }

            if columns[1].button("Impulsion").clicked() {
                self.impulse();
            }
        });

        ui.columns(2, |columns| {
            columns[0].label("Valeur y maximale:");
            columns[1].add(egui::DragValue::new(&mut self.max_y).speed(0.01));
        });

        if let Some(idx) = self.selected_point {
            ui.collapsing(format!("Point sélectionné: {idx}"), |ui| {
                if ui.button("Épingler").clicked() {
                    self.pin_selected_point(renderer);
                }
            });
        }
        {
            let mut model = self.model.lock().unwrap();
            let determinant = model.minimum_determinant(&self.solver.x);
            if determinant < 0.0 {
                ui.colored_label(Color32::RED, format!("min det F = {}", determinant));
            } else {
                ui.label(format!("min det F = {}", determinant));
            }
            model.ui(ui, renderer);
        }
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
