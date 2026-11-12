use nalgebra::{Matrix3, Vector3};

use super::LameParameters;

fn corotational_tensor(F: &Matrix3<f64>, R: &Matrix3<f64>) -> Matrix3<f64> {
    R.transpose() * F - Matrix3::identity()
}

pub fn energy_density(F: &Matrix3<f64>, lame: &LameParameters) -> f64 {
    let (R, _) = F.polar();

    let lhs = lame.mu * (F - R).norm_squared();
    let rhs = 0.5 * lame.lambda * corotational_tensor(F, &R).trace().powi(2);

    lhs + rhs
}

pub fn compute_P(F: &Matrix3<f64>, lame: &LameParameters) -> Matrix3<f64> {
    let (R, _) = F.polar();

    let lhs = 2.0 * lame.mu * (F - R);
    let rhs = lame.lambda * corotational_tensor(F, &R).trace() * R;

    lhs + rhs
}

#[rustfmt::skip]
fn skew(vec: &Vector3<f64>) -> Matrix3<f64> {
    Matrix3::new(
        0.0, -vec[2], vec[1],
        vec[2], 0.0, -vec[0],
        -vec[1], vec[0], 0.0
    )
}

fn unskew(mat: &Matrix3<f64>) -> Vector3<f64> {
    0.5 * Vector3::new(
        mat[(2, 1)] - mat[(1, 2)],
        mat[(0, 2)] - mat[(2, 0)],
        mat[(1, 0)] - mat[(0, 1)],
    )
}

// src: https://julianpanetta.com/pdf/notes/PolarDecompositionDerivative.pdf
fn compute_delta_R(R: &Matrix3<f64>, delta_F: &Matrix3<f64>, S: &Matrix3<f64>) -> Matrix3<f64> {
    let A = S.trace() * Matrix3::identity() - S;
    let rhs = 2.0 * unskew(&(R.transpose() * delta_F));
    let X = A.lu().solve(&rhs).unwrap();

    R * skew(&X)
}

pub fn compute_delta_P(
    F: &Matrix3<f64>,
    delta_F: &Matrix3<f64>,
    lame: &LameParameters,
) -> Matrix3<f64> {
    let (R, S) = F.polar();
    let delta_R = compute_delta_R(&R, delta_F, &S);

    let delta_corotational_tensor = delta_R.transpose() * F + R.transpose() * delta_F;

    let delta_lhs = 2.0 * lame.mu * (delta_F - delta_R);
    let delta_rhs = lame.lambda * corotational_tensor(F, &R).trace() * delta_R
        + lame.lambda * delta_corotational_tensor.trace() * R;

    delta_lhs + delta_rhs
}
