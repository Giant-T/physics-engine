use std::rc::Rc;

use nalgebra::DVector;
use nalgebra_sparse::{CooMatrix, CsrMatrix};

use crate::physics::{linalg::ConjugateGradient, tet_mesh::TetMesh};

use super::elastic_model::ElasticModel;

pub struct SolverOptions {
    time_step: f64,
    max_iterations: usize,
    tolerance: f64,
}

impl Default for SolverOptions {
    fn default() -> Self {
        Self {
            time_step: 0.02,
            max_iterations: 2000,
            tolerance: 1e-10,
        }
    }
}

pub struct ImplicitSolver {
    model: Rc<ElasticModel>,
    time_step: f64,
    last_iterations: usize,
    last_residual: f64,
    conjugate_gradient: ConjugateGradient,
}

impl ImplicitSolver {
    pub fn new(model: Rc<ElasticModel>, options: SolverOptions) -> Self {
        Self {
            model,
            time_step: options.time_step,
            conjugate_gradient: ConjugateGradient::new(options.max_iterations, options.tolerance),
            last_iterations: 0,
            last_residual: 0.0,
        }
    }

    pub fn assemble_system(
        &self,
        x: &DVector<f64>,
        v: &DVector<f64>,
        h: f64,
    ) -> (CsrMatrix<f64>, DVector<f64>) {
        let system_matrix = self.system_matrix(x, h);

        let stiffness = self.model.stiffness_matrix(x);
        let f = self.model.compute_forces_with_stiffness(x, v, &stiffness);
        let rhs = h * (f - h * stiffness * v);

        (system_matrix, rhs)
    }

    pub fn system_matrix(&self, x: &DVector<f64>, h: f64) -> CsrMatrix<f64> {
        let stiffness = self.model.stiffness_matrix(x);

        let mass_factor = 1.0 + h * self.model.rayleigh_alpha();
        let stiffness_factor = h.powi(2) + h * self.model.rayleigh_beta();

        let n = stiffness.nrows();
        let mut system_matrix = CooMatrix::new(n, n);

        for (i, j, val) in stiffness.triplet_iter() {
            system_matrix.push(i, j, stiffness_factor * val);
        }

        for (i, val) in self.model.lumped_masses().iter().enumerate() {
            system_matrix.push(i, i, mass_factor * val);
        }

        CsrMatrix::from(&system_matrix)
    }

    pub fn step(&mut self, x: &mut DVector<f64>, v: &mut DVector<f64>) {
        let h = self.time_step;
        let (system_matrix, rhs) = self.assemble_system(x, v, h);
        Self::apply_pin_constraints(self.model.mesh(), &system_matrix, &rhs);
        let delta_v = self.solve(&system_matrix, &rhs);

        *v += delta_v;
        *x += h * v.clone();
    }

    fn apply_pin_constraints(mesh: &TetMesh, system_matrix: &CsrMatrix<f64>, rhs: &DVector<f64>) {
        todo!();
    }

    fn solve(&mut self, system_matrix: &CsrMatrix<f64>, rhs: &DVector<f64>) -> DVector<f64> {
        self.conjugate_gradient.solve(system_matrix, rhs)
    }
}
