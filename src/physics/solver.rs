use std::{ops::AddAssign, rc::Rc, sync::Mutex};

use nalgebra::{DVector, Vector3};
use nalgebra_sparse::{CooMatrix, CsrMatrix};

use crate::renderer::UIElement;

use super::{elastic_model::ElasticModel, linalg::ConjugateGradient, tet_mesh::TetMesh};

pub struct SolverOptions {
    pub time_step: f64,
    pub max_iterations: usize,
    pub tolerance: f64,
}

impl Default for SolverOptions {
    fn default() -> Self {
        Self {
            time_step: 0.02,
            max_iterations: 1000,
            tolerance: 1e-5,
        }
    }
}

pub struct ImplicitSolver {
    model: Rc<Mutex<ElasticModel>>,
    time_step: f64,
    conjugate_gradient: ConjugateGradient,
    pub x: DVector<f64>,
    pub v: DVector<f64>,
    delta_v: DVector<f64>,
}

impl ImplicitSolver {
    pub fn new(model: Rc<Mutex<ElasticModel>>, options: SolverOptions) -> Self {
        log::info!("ImplicitSolver::new");

        let binding = model.clone();
        let model_val = binding.lock().unwrap();
        let rest_positions = model_val.mesh().rest_positions();
        let x = DVector::from_iterator(
            rest_positions.len() * 3,
            rest_positions.iter().flat_map(|v| v.iter().copied()),
        );
        let v = DVector::zeros(model_val.mesh().dof_count());
        let delta_v = DVector::zeros(model_val.mesh().dof_count());
        Self {
            model,
            time_step: options.time_step,
            conjugate_gradient: ConjugateGradient::new(options.max_iterations, options.tolerance),
            x,
            v,
            delta_v,
        }
    }

    pub fn assemble_system(&self, h: f64) -> (CsrMatrix<f64>, DVector<f64>) {
        let stiffness = self.model.lock().unwrap().stiffness_matrix(&self.x).clone();
        let system_matrix = self.system_matrix(h, &stiffness);

        let f = self
            .model
            .lock()
            .unwrap()
            .compute_forces_with_stiffness(&self.x, &self.v, &stiffness);
        let rhs = h * (f - h * stiffness * &self.v);

        (system_matrix, rhs)
    }

    pub fn system_matrix(&self, h: f64, stiffness: &CsrMatrix<f64>) -> CsrMatrix<f64> {
        let model = self.model.lock().unwrap();
        let mass_factor = 1.0 + h * model.rayleigh_alpha();
        let stiffness_factor = h.powi(2) + h * model.rayleigh_beta();

        let n = stiffness.nrows();
        let mut system_matrix = CooMatrix::new(n, n);

        for (i, j, val) in stiffness.triplet_iter() {
            system_matrix.push(i, j, stiffness_factor * val);
        }

        for (i, val) in model.lumped_masses().iter().enumerate() {
            system_matrix.push(i, i, mass_factor * val);
        }

        CsrMatrix::from(&system_matrix)
    }

    pub fn step(&mut self) {
        let h = self.time_step;
        let (mut system_matrix, mut rhs) = self.assemble_system(h);

        Self::apply_pin_constraints(
            self.model.lock().unwrap().mesh(),
            &mut system_matrix,
            &mut rhs,
        );

        self.delta_v = self.solve(&system_matrix, &rhs);

        self.v += &self.delta_v;
        self.x += h * &self.v;
    }

    fn apply_pin_constraints(
        mesh: &TetMesh,
        system_matrix: &mut CsrMatrix<f64>,
        rhs: &mut DVector<f64>,
    ) {
        let mut pinned = vec![false; system_matrix.nrows()];
        for i in 0..mesh.node_count() {
            if mesh.is_pinned(i) {
                pinned[(3 * i)..(3 * i + 3)].fill(true);
            }
        }

        for (row, col, val) in system_matrix.triplet_iter_mut() {
            if pinned[row] || pinned[col] {
                *val = if row == col { 1.0 } else { 0.0 };
            }
        }

        for i in 0..rhs.len() {
            if pinned[i] {
                rhs[i] = 0.0;
            }
        }
    }

    fn solve(&mut self, system_matrix: &CsrMatrix<f64>, rhs: &DVector<f64>) -> DVector<f64> {
        self.conjugate_gradient
            .solve(system_matrix, rhs, &self.delta_v)
    }

    pub fn reset(&mut self) {
        let model = self.model.lock().unwrap();
        self.x = DVector::from_iterator(
            model.mesh().rest_positions().len() * 3,
            model
                .mesh()
                .rest_positions()
                .iter()
                .flat_map(|v| v.iter().copied()),
        );
        self.v = DVector::zeros(model.mesh().dof_count());
        self.delta_v = DVector::zeros(model.mesh().dof_count());
    }

    pub fn current_positions(&self) -> Vec<Vector3<f64>> {
        let current_positions: Vec<Vector3<f64>> = self
            .x
            .as_slice()
            .chunks_exact(3)
            .map(|v| Vector3::new(v[0], v[1], v[2]))
            .collect();

        current_positions
    }

    fn impulse(&mut self) {
        let model = self.model.lock().unwrap();
        let impulse = Vector3::new(1.5, 0.0, 0.4);
        for idx in 0..model.mesh().node_count() {
            if !model.mesh().is_pinned(idx) {
                self.v.fixed_rows_mut::<3>(3 * idx).add_assign(&impulse);
            }
        }
    }
}

impl UIElement for ImplicitSolver {
    fn ui(&mut self, ui: &mut egui::Ui) {
        if ui.button("Impulsion").clicked() {
            self.impulse();
        }
    }
}
