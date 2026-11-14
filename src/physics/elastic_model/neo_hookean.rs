use nalgebra::Matrix3;

use super::LameParameters;

// src: https://www.tkim.graphics/NEO/StableNeoHookean2018.pdf

fn compute_alpha(lame: &LameParameters) -> f64 {
    1.0 + lame.mu / lame.lambda - lame.mu / (4.0 * lame.lambda)
}

fn compute_Ic(F: &Matrix3<f64>) -> f64 {
    (F.transpose() * F).trace()
}

fn compute_cofactor(F: &Matrix3<f64>) -> Matrix3<f64> {
    Matrix3::from_columns(&[
        F.column(1).cross(&F.column(2)),
        F.column(2).cross(&F.column(0)),
        F.column(0).cross(&F.column(1)),
    ])
}

fn compute_delta_cofactor(F: &Matrix3<f64>, delta_F: &Matrix3<f64>) -> Matrix3<f64> {
    Matrix3::from_columns(&[
        delta_F.column(1).cross(&F.column(2)) + F.column(1).cross(&delta_F.column(2)),
        delta_F.column(2).cross(&F.column(0)) + F.column(2).cross(&delta_F.column(0)),
        delta_F.column(0).cross(&F.column(1)) + F.column(0).cross(&delta_F.column(1)),
    ])
}

pub fn energy_density(F: &Matrix3<f64>, lame: &LameParameters) -> f64 {
    let J = F.determinant();
    let alpha = compute_alpha(lame);
    let Ic = compute_Ic(F);

    lame.mu / 2.0 * (Ic - 3.0) - lame.mu * (J - alpha).powi(2) + lame.lambda / 2.0 * (J - 1.0).ln()
}

pub fn compute_P(F: &Matrix3<f64>, lame: &LameParameters) -> Matrix3<f64> {
    let J = F.determinant();
    let Ic = compute_Ic(F);
    let alpha = compute_alpha(lame);
    let cofactor = compute_cofactor(F);

    let lhs = lame.mu * (1.0 - 1.0 / (Ic - 1.0)) * F;
    let rhs = lame.lambda * (J - alpha) * cofactor;

    lhs + rhs
}

pub fn compute_delta_P(
    F: &Matrix3<f64>,
    delta_F: &Matrix3<f64>,
    lame: &LameParameters,
) -> Matrix3<f64> {
    let J = F.determinant();
    let alpha = compute_alpha(lame);
    let Ic = compute_Ic(F);
    let cofactor = compute_cofactor(F);

    let Tikhonov = delta_F;
    let Mu = (F.transpose() * delta_F).trace() * F;
    let Gradient = (cofactor.transpose() * delta_F).trace() * cofactor;
    let VolumeHessian = compute_delta_cofactor(F, delta_F);

    lame.mu * (1.0 - 1.0 / (Ic - 1.0)) * Tikhonov
        + lame.mu * 2.0 / (Ic + 1.0).powi(2) * Mu
        + lame.lambda * Gradient
        + lame.lambda * (J - alpha) * VolumeHessian
}
