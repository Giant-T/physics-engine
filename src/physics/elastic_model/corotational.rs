use nalgebra::{Matrix3, Vector3};

use super::LameParameters;

// Generer par IA TODO: Refaire la fonction
pub fn polar_decomposition(F: &Matrix3<f64>) -> (Matrix3<f64>, Matrix3<f64>) {
    let svd = F.svd(true, true);

    let u = svd.u.expect("SVD U matrix missing");
    let v_t = svd.v_t.expect("SVD V_t matrix missing");
    let v = v_t.transpose();

    // R = U * V^T
    let mut r = u * v.transpose();

    // Correction cruciale pour éviter les réflexions (det(R) == -1)
    if r.determinant() < 0.0 {
        // On inverse le signe de la dernière colonne de U et de la dernière valeur singulière
        let mut u_corrected = u;
        u_corrected.column_mut(2).scale_mut(-1.0);
        r = u_corrected * v.transpose();
    }

    // S = V * Lambda * V^T ou calculé via F
    // Pour le corotationnel, S est souvent obtenu par R.transpose() * F
    let s = r.transpose() * F;

    (r, s)
}

fn corotational_tensor(F: &Matrix3<f64>, R: &Matrix3<f64>) -> Matrix3<f64> {
    R.transpose() * F - Matrix3::identity()
}

pub fn energy_density(F: &Matrix3<f64>, lame: &LameParameters) -> f64 {
    let (R, _) = polar_decomposition(F);

    let lhs = lame.mu * (F - R).norm_squared();
    let rhs = 0.5 * lame.lambda * corotational_tensor(F, &R).trace().powi(2);

    lhs + rhs
}

pub fn compute_P(F: &Matrix3<f64>, lame: &LameParameters) -> Matrix3<f64> {
    let (R, _) = polar_decomposition(F);

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
fn compute_delta_R(R: &Matrix3<f64>, delta_F: &Matrix3<f64>, U: &Matrix3<f64>) -> Matrix3<f64> {
    let A = U.trace() * Matrix3::identity() - U;
    let rhs = 2.0 * unskew(&(R.transpose() * delta_F));
    let X = A.lu().solve(&rhs).unwrap();

    R * skew(&X)
}

pub fn compute_delta_P(
    F: &Matrix3<f64>,
    delta_F: &Matrix3<f64>,
    lame: &LameParameters,
) -> Matrix3<f64> {
    let (R, U) = polar_decomposition(F);
    let delta_R = compute_delta_R(&R, delta_F, &U);

    let delta_corotational_tensor = delta_R.transpose() * F + R.transpose() * delta_F;

    let delta_lhs = 2.0 * lame.mu * (delta_F - delta_R);
    let delta_rhs = lame.lambda * corotational_tensor(F, &R).trace() * delta_R
        + lame.lambda * delta_corotational_tensor.trace() * R;

    delta_lhs + delta_rhs
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Rotation3;

    #[test]
    fn test_rest_state() {
        let lame = LameParameters::new(1000.0, 1000.0);
        let F = Matrix3::identity();

        let P = compute_P(&F, &lame);

        assert!(
            P.norm() < 1e-10,
            "Le tenseur P n'est pas nul au repos ! P = {P}"
        );
    }

    #[test]
    fn test_rigid_rotation() {
        let lame = LameParameters::new(1000.0, 1000.0);

        let angle = 0.5;
        let F = Rotation3::from_axis_angle(&Vector3::z_axis(), angle)
            .matrix()
            .clone();
        let P = compute_P(&F, &lame);

        assert!(
            P.norm() < 1e-10,
            "Echec du test de rotation rigide: P genere des forces parasites P = {P}",
        );
    }
}
