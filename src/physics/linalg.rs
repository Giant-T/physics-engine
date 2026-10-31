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
        Self {
            max_iterations,
            tolerance,
            iterations: 0,
            residual: 0.0,
        }
    }

    // TODO: Changer le type de retour pour ne pas avoir a consulter les variables membres de LinearSolver
    pub fn solve(&mut self, matrix: &CsrMatrix<f64>, rhs: &DVector<f64>) -> DVector<f64> {
        self.iterations = 0;
        self.residual = 0.0;
        let rhs_norm = rhs.norm();

        // norm(rhs) = 0
        if rhs_norm <= f64::EPSILON {
            return DVector::zeros(rhs.nrows());
        }

        let mut x = DVector::zeros(rhs.nrows());
        let mut r = rhs - matrix * &x;

        if r.norm() / rhs_norm <= self.tolerance {
            return x;
        }

        let mut p = r.clone();

        for k in 0..self.max_iterations {
            self.iterations = k + 1;

            let A_p = matrix * &p;
            let alpha = r.dot(&r) / p.dot(&A_p);

            x += alpha * &p;
            let old_r = r.clone();
            r -= alpha * &A_p;

            if r.norm() / rhs_norm <= self.tolerance {
                break;
            }

            let beta = r.dot(&r) / old_r.dot(&old_r);
            p = &r + beta * p;
        }

        self.residual = r.norm();
        x
    }

    pub fn iterations(&self) -> usize {
        self.iterations
    }

    pub fn residual(&self) -> f64 {
        self.residual
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
        let x = solver.solve(&a, &b);

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
        let x = solver.solve(&a, &b);

        assert!((&x - &expected).norm() < 1e-8);
    }

    #[test]
    fn solve_zero_rhs() {
        let a = create_spd_matrix();
        let b = DVector::zeros(3);

        let mut solver = ConjugateGradient::new(100, 1e-10);
        let x = solver.solve(&a, &b);

        assert!(x.norm() < 1e-10);
        assert_eq!(solver.iterations(), 0);
    }

    #[test]
    fn solution_satisfies_linear_system() {
        let a = create_spd_matrix();
        let b = dvector![6.0, 10.0, 8.0];

        let mut solver = ConjugateGradient::new(100, 1e-10);
        let x = solver.solve(&a, &b);

        let residual = &a * &x - &b;

        assert!(
            residual.norm() < 1e-8,
            "Linear system residual too large: {}",
            residual.norm()
        );
    }
}
