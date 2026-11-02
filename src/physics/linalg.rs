use nalgebra::DVector;
use nalgebra_sparse::CsrMatrix;

pub struct ConjugateGradient {
    max_iterations: usize,
    tolerance: f64,
    iterations: usize,
    residual: f64,
}

impl ConjugateGradient {
    pub fn new(max_iterations: usize, tolerance: f64) -> Self {
        log::info!("ConjugateGradient::new");

        Self {
            max_iterations,
            tolerance,
            iterations: 0,
            residual: 0.0,
        }
    }

    // TODO: Changer le type de retour pour ne pas avoir a consulter les variables membres de LinearSolver
    // TODO: Garder un oeil sur la performance si le guess initial est mauvais
    pub fn solve(
        &mut self,
        matrix: &CsrMatrix<f64>,
        rhs: &DVector<f64>,
        initial_guess: &DVector<f64>,
    ) -> DVector<f64> {
        self.iterations = 0;
        self.residual = 0.0;
        let rhs_norm = rhs.norm();

        // norm(rhs) = 0
        if rhs_norm <= f64::EPSILON {
            return DVector::zeros(rhs.nrows());
        }

        let mut x = initial_guess.clone();
        let mut r = rhs - matrix * &x;

        if r.norm() / rhs_norm <= self.tolerance {
            return x;
        }

        let inverse_diagonal = Self::inverse_diagonal(matrix);
        let mut z = inverse_diagonal.component_mul(&r);
        let mut p = z.clone();

        let mut rz = r.dot(&z);

        for k in 0..self.max_iterations {
            self.iterations = k + 1;

            let matrix_p = matrix * &p;
            let alpha = rz / p.dot(&matrix_p);

            x.axpy(alpha, &p, 1.0);
            r.axpy(-alpha, &matrix_p, 1.0);

            if r.norm() / rhs_norm <= self.tolerance {
                break;
            }

            z.copy_from(&r);
            z.component_mul_assign(&inverse_diagonal);

            let new_rz = r.dot(&z);
            let beta = new_rz / rz;

            p.scale_mut(beta);
            p += &z;
            rz = new_rz;
        }

        self.residual = r.norm() / rhs_norm;
        x
    }

    fn inverse_diagonal(matrix: &CsrMatrix<f64>) -> DVector<f64> {
        let mut diagonal = DVector::zeros(matrix.nrows());

        for (row, col, val) in matrix.triplet_iter() {
            if row == col {
                diagonal[row] = 1.0 / val;
            }
        }

        diagonal
    }

    pub fn iterations(&self) -> usize {
        self.iterations
    }

    pub fn residual(&self) -> f64 {
        self.residual
    }

    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
}

// TODO: Reverifier les tests
#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::dvector;
    use nalgebra_sparse::coo::CooMatrix;

    fn create_spd_matrix() -> CsrMatrix<f64> {
        // A = [ 4  1  0 ]
        //     [ 1  3  1 ]
        //     [ 0  1  2 ]

        let mut coo = CooMatrix::new(3, 3);

        coo.push(0, 0, 4.0);
        coo.push(0, 1, 1.0);

        coo.push(1, 0, 1.0);
        coo.push(1, 1, 3.0);
        coo.push(1, 2, 1.0);

        coo.push(2, 1, 1.0);
        coo.push(2, 2, 2.0);

        CsrMatrix::from(&coo)
    }

    #[test]
    fn solve_spd_system() {
        let a = create_spd_matrix();

        let b = dvector![6.0, 10.0, 8.0];
        let expected = dvector![1.0, 2.0, 3.0];

        let mut solver = ConjugateGradient::new(100, 1e-10);
        let x = solver.solve(&a, &b, &dvector![0.0, 0.0, 0.0]);

        assert!(
            (&x - &expected).norm() < 1e-8,
            "Expected {:?}, got {:?}",
            expected,
            x
        );

        assert!(solver.residual() < 1e-8);
    }

    #[test]
    fn solve_diagonal_system() {
        let mut coo = CooMatrix::new(3, 3);

        coo.push(0, 0, 2.0);
        coo.push(1, 1, 4.0);
        coo.push(2, 2, 8.0);

        let a = CsrMatrix::from(&coo);

        let b = dvector![6.0, -8.0, 4.0];
        let expected = dvector![3.0, -2.0, 0.5];

        let mut solver = ConjugateGradient::new(100, 1e-10);
        let x = solver.solve(&a, &b, &dvector![0.0, 0.0, 0.0]);

        assert!((&x - &expected).norm() < 1e-8);
    }

    #[test]
    fn solve_zero_rhs() {
        let a = create_spd_matrix();
        let b = DVector::zeros(3);

        let mut solver = ConjugateGradient::new(100, 1e-10);
        let x = solver.solve(&a, &b, &dvector![0.0, 0.0, 0.0]);

        assert!(x.norm() < 1e-10);
        assert_eq!(solver.iterations(), 0);
    }

    #[test]
    fn solution_satisfies_linear_system() {
        let a = create_spd_matrix();
        let b = dvector![6.0, 10.0, 8.0];

        let mut solver = ConjugateGradient::new(100, 1e-10);
        let x = solver.solve(&a, &b, &dvector![0.0, 0.0, 0.0]);

        let residual = &a * &x - &b;

        assert!(
            residual.norm() < 1e-8,
            "Linear system residual too large: {}",
            residual.norm()
        );
    }
}
