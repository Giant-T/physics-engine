use nalgebra::Matrix3;

use super::elastic_model::LameParameters;

type Tensor = Matrix3<f64>;

fn green_strain(F: &Tensor) -> Tensor {
    0.5 * (F.transpose() * F - Tensor::identity())
}

pub fn energy_density(F: &Tensor, lame: &LameParameters) -> f64 {
    let E = green_strain(F);

    let trace = E.trace();

    lame.mu * E.norm_squared() + 0.5 * lame.lambda * trace.powi(2)
}

fn second_piola_kirchhoff(F: &Matrix3<f64>, lame: &LameParameters) -> Matrix3<f64> {
    let E = green_strain(F);

    2.0 * lame.mu * E + lame.lambda * E.trace() * Matrix3::identity()
}

pub fn first_piola_kirchhoff(F: &Matrix3<f64>, lame: &LameParameters) -> Matrix3<f64> {
    return F * second_piola_kirchhoff(F, lame);
}

pub fn delta_first_piola_kirchhoff(
    F: &Matrix3<f64>,
    delta_F: &Matrix3<f64>,
    lame: &LameParameters,
) -> Matrix3<f64> {
    let S = second_piola_kirchhoff(F, lame);

    let delta_E = 0.5 * (delta_F.transpose() * F + F.transpose() * delta_F);
    let delta_S = 2.0 * lame.mu * delta_E + lame.lambda * delta_E.trace() * Matrix3::identity();

    delta_F * S + F * delta_S
}
