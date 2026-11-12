use nalgebra::Matrix3;

use super::{LameParameters, corotational, stvk};

#[derive(Debug, PartialEq)]
pub enum ElasticModelType {
    StVK,
    Corotationnal,
}

impl ElasticModelType {
    pub fn energy_density(&self, F: &Matrix3<f64>, lame: &LameParameters) -> f64 {
        match self {
            Self::StVK => stvk::energy_density(F, lame),
            Self::Corotationnal => corotational::energy_density(F, lame),
        }
    }

    pub fn compute_P(&self, F: &Matrix3<f64>, lame: &LameParameters) -> Matrix3<f64> {
        match self {
            Self::StVK => stvk::first_piola_kirchhoff(F, lame),
            Self::Corotationnal => corotational::compute_P(F, lame),
        }
    }

    pub fn compute_delta_P(
        &self,
        F: &Matrix3<f64>,
        delta_F: &Matrix3<f64>,
        lame: &LameParameters,
    ) -> Matrix3<f64> {
        match self {
            Self::StVK => stvk::delta_first_piola_kirchhoff(F, delta_F, lame),
            Self::Corotationnal => corotational::compute_delta_P(F, delta_F, lame),
        }
    }
}
