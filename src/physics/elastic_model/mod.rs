use std::ops::{AddAssign, SubAssign};

use nalgebra::{DVector, Matrix3, RowVector3, SMatrix, SVector, Vector3};
use nalgebra_sparse::{CooMatrix, CsrMatrix};

use crate::renderer::UIElement;

use super::{GRAVITY, tet_mesh::TetMesh};
use model_type::ElasticModelType;

mod corotational;
pub mod model_type;
mod stvk;

pub struct LameParameters {
    pub mu: f64,
    pub lambda: f64,
}

impl LameParameters {
    pub fn new(mu: f64, lambda: f64) -> Self {
        Self { mu, lambda }
    }

    pub fn from_young_poisson(young: f64, poisson: f64) -> Self {
        debug_assert!(young > 0.0);
        debug_assert!(poisson > -1.0 && poisson < 0.5);

        Self {
            mu: young / (2.0 * (1.0 + poisson)),
            lambda: young * poisson / ((1.0 + poisson) * (1.0 - 2.0 * poisson)),
        }
    }
}

pub struct ElasticModel {
    mesh: TetMesh,
    lame: LameParameters,
    lumped_masses: DVector<f64>,
    total_mass: f64,
    rayleigh_alpha: f64,
    rayleigh_beta: f64,
    damping: CsrMatrix<f64>,
    model_type: ElasticModelType,
}

impl ElasticModel {
    pub fn new(
        mesh: TetMesh,
        lame: LameParameters,
        density: f64,
        model_type: ElasticModelType,
    ) -> Self {
        log::info!("ElasticModel::new");

        let (lumped_masses, total_mass) = Self::build_masses(&mesh, density);
        let rayleigh_alpha = 0.0;
        let rayleigh_beta = 0.0;

        let rest_positions = mesh.rest_positions();
        let x = DVector::from_iterator(
            rest_positions.len() * 3,
            rest_positions.iter().flat_map(|v| v.iter().copied()),
        );
        let stiffness = Self::compute_stiffness(&model_type, &mesh, &lame, &x);
        let damping =
            Self::initialize_damping(&lumped_masses, &stiffness, rayleigh_alpha, rayleigh_beta);

        Self {
            mesh,
            lame,
            lumped_masses,
            total_mass,
            damping,
            rayleigh_alpha,
            rayleigh_beta,
            model_type,
        }
    }

    fn build_masses(mesh: &TetMesh, density: f64) -> (DVector<f64>, f64) {
        let mut lumped_masses = DVector::zeros(mesh.dof_count());
        let mut total_mass = 0.0;

        for e in 0..mesh.tet_count() {
            let rest_volume = mesh.elements()[e].rest_volume();
            let mass = density * rest_volume;
            let node_mass = mass / 4.0;

            total_mass += mass;

            for idx in &mesh.tets()[e] {
                lumped_masses.rows_mut(idx * 3, 3).add_scalar_mut(node_mass);
            }
        }

        debug_assert!(lumped_masses.iter().all(|m| *m > 0.0));

        (lumped_masses, total_mass)
    }

    pub fn elastic_energy(&self, x: &DVector<f64>) -> f64 {
        let mut energy = 0.0;

        for e in 0..self.mesh.tet_count() {
            let tet = &self.mesh.tets()[e];
            let el = &self.mesh.elements()[e];

            let F = el.deformation_gradient(
                &x.fixed_rows::<3>(3 * tet[0]).into(),
                &x.fixed_rows::<3>(3 * tet[1]).into(),
                &x.fixed_rows::<3>(3 * tet[2]).into(),
                &x.fixed_rows::<3>(3 * tet[3]).into(),
            );

            energy += el.rest_volume() * self.model_type.energy_density(&F, &self.lame);
        }

        energy
    }

    fn potential_energy(&self, x: &DVector<f64>) -> f64 {
        let mut energy = self.elastic_energy(x);

        for i in 0..self.mesh.node_count() {
            if self.mesh.is_pinned(i) {
                continue;
            }
            energy -= self.lumped_masses[3 * i] * GRAVITY.dot(&x.fixed_rows::<3>(3 * i));
        }

        energy
    }

    pub fn kinetic_energy(&self, v: &DVector<f64>) -> f64 {
        let mut energy = 0.0;

        for i in 0..self.mesh.node_count() {
            if self.mesh.is_pinned(i) {
                continue;
            }
            energy += 0.5 * self.lumped_masses[3 * i] * v.fixed_rows::<3>(3 * i).norm_squared();
        }

        energy
    }

    pub fn total_energy(&self, x: &DVector<f64>, v: &DVector<f64>) -> f64 {
        self.kinetic_energy(v) + self.potential_energy(x)
    }

    pub fn add_element_forces(&self, element_idx: usize, x: &DVector<f64>, f: &mut DVector<f64>) {
        let tet = &self.mesh.tets()[element_idx];
        let element = &self.mesh.elements()[element_idx];

        let p0 = x.fixed_rows::<3>(3 * tet[0]);
        let p1 = x.fixed_rows::<3>(3 * tet[1]);
        let p2 = x.fixed_rows::<3>(3 * tet[2]);
        let p3 = x.fixed_rows::<3>(3 * tet[3]);

        let F = element.deformation_gradient(&p0.into(), &p1.into(), &p2.into(), &p3.into());
        let P = self.model_type.compute_P(&F, &self.lame);

        let H = -element.rest_volume() * P * element.rest_inverse().transpose();

        let h0 = H.column(0);
        let h1 = H.column(1);
        let h2 = H.column(2);

        f.fixed_rows_mut::<3>(3 * tet[1]).add_assign(h0);
        f.fixed_rows_mut::<3>(3 * tet[2]).add_assign(h1);
        f.fixed_rows_mut::<3>(3 * tet[3]).add_assign(h2);
        f.fixed_rows_mut::<3>(3 * tet[0]).sub_assign(h0 + h1 + h2);
    }

    pub fn elastic_forces(&self, x: &DVector<f64>) -> DVector<f64> {
        let mut f = DVector::zeros(self.mesh.dof_count());

        for e in 0..self.mesh.tet_count() {
            self.add_element_forces(e, x, &mut f);
        }

        f
    }

    pub fn compute_forces_with_stiffness(
        &mut self,
        x: &DVector<f64>,
        v: &DVector<f64>,
        stiffness: &CsrMatrix<f64>,
    ) -> DVector<f64> {
        let mut f = self.elastic_forces(x);

        for i in 0..self.mesh.node_count() {
            f.fixed_rows_mut::<3>(3 * i)
                .add_assign(self.lumped_masses[3 * i] * GRAVITY);
        }

        if self.rayleigh_alpha != 0.0 || self.rayleigh_beta != 0.0 {
            f -= self.damping_matrix(stiffness) * v;
        }

        for i in 0..self.mesh.node_count() {
            if self.mesh.is_pinned(i) {
                f.fixed_rows_mut::<3>(i * 3).copy_from(&Vector3::zeros());
            }
        }

        f
    }
    fn model_element_stiffness(
        &self,
        element_idx: usize,
        x: &DVector<f64>,
    ) -> SMatrix<f64, 12, 12> {
        Self::element_stiffness(&self.model_type, &self.mesh(), &self.lame, element_idx, x)
    }

    fn element_stiffness(
        model_type: &ElasticModelType,
        mesh: &TetMesh,
        lame: &LameParameters,
        element_idx: usize,
        x: &DVector<f64>,
    ) -> SMatrix<f64, 12, 12> {
        let tet = &mesh.tets()[element_idx];
        let element = &mesh.elements()[element_idx];

        let F = element.deformation_gradient(
            &x.fixed_rows::<3>(3 * tet[0]).into(),
            &x.fixed_rows::<3>(3 * tet[1]).into(),
            &x.fixed_rows::<3>(3 * tet[2]).into(),
            &x.fixed_rows::<3>(3 * tet[3]).into(),
        );

        let rest_inverse = element.rest_inverse();
        let volume = element.rest_volume();

        let mut stiffness = SMatrix::<f64, 12, 12>::zeros();

        for local_node in 0..4 {
            for component in 0..3 {
                let mut delta_shape = Matrix3::<f64>::zeros();

                if local_node == 0 {
                    delta_shape.set_row(component, &RowVector3::repeat(-1.0));
                } else {
                    delta_shape[(component, local_node - 1)] = 1.0;
                }

                let delta_F = delta_shape * rest_inverse;
                let delta_P = model_type.compute_delta_P(&F, &delta_F, lame);
                let delta_H = -volume * delta_P * rest_inverse.transpose();

                let mut force_differential = SVector::<f64, 12>::zeros();
                let mut sum = Vector3::<f64>::zeros();

                for k in 0..3 {
                    force_differential
                        .fixed_rows_mut::<3>(3 * (k + 1))
                        .copy_from(&delta_H.column(k));
                    sum += delta_H.column(k);
                }
                force_differential.fixed_rows_mut::<3>(0).copy_from(&-sum);

                stiffness.set_column(3 * local_node + component, &-force_differential);
            }
        }

        stiffness
    }

    fn compute_stiffness(
        model_type: &ElasticModelType,
        mesh: &TetMesh,
        lame: &LameParameters,
        x: &DVector<f64>,
    ) -> CsrMatrix<f64> {
        let mut matrix = CooMatrix::<f64>::new(mesh.dof_count(), mesh.dof_count());

        for e in 0..mesh.tet_count() {
            let tet = &mesh.tets()[e];
            let local = Self::element_stiffness(model_type, mesh, lame, e, &x);

            for a in 0..tet.len() {
                for b in 0..tet.len() {
                    for i in 0..3 {
                        for j in 0..3 {
                            let val = local[(3 * a + i, 3 * b + j)];
                            matrix.push(3 * tet[a] + i, 3 * tet[b] + j, val);
                        }
                    }
                }
            }
        }

        CsrMatrix::from(&matrix)
    }

    pub fn stiffness_matrix(&self, x: &DVector<f64>) -> CsrMatrix<f64> {
        Self::compute_stiffness(&self.model_type, &self.mesh, &self.lame, x)
    }

    fn initialize_damping(
        lumped_masses: &DVector<f64>,
        stiffness: &CsrMatrix<f64>,
        rayleigh_alpha: f64,
        rayleigh_beta: f64,
    ) -> CsrMatrix<f64> {
        let n = lumped_masses.len();
        let mut matrix = CooMatrix::new(n, n);

        if rayleigh_alpha != 0.0 {
            for (i, val) in lumped_masses.iter().enumerate() {
                matrix.push(i, i, rayleigh_alpha * val);
            }
        }

        if rayleigh_beta != 0.0 {
            for (i, j, val) in stiffness.triplet_iter() {
                matrix.push(i, j, rayleigh_beta * val);
            }
        }

        CsrMatrix::from(&matrix)
    }

    pub fn damping_matrix(&mut self, stiffness: &CsrMatrix<f64>) -> &CsrMatrix<f64> {
        for (i, j, val) in self.damping.triplet_iter_mut() {
            *val = self.rayleigh_beta * stiffness.get_entry(i, j).unwrap().into_value();
            if i == j {
                *val += self.rayleigh_alpha * self.lumped_masses[i];
            }
        }

        &self.damping
    }

    pub fn rayleigh_alpha(&self) -> f64 {
        self.rayleigh_alpha
    }

    pub fn rayleigh_beta(&self) -> f64 {
        self.rayleigh_beta
    }

    pub fn lumped_masses(&self) -> &DVector<f64> {
        &self.lumped_masses
    }

    pub fn mesh(&self) -> &TetMesh {
        &self.mesh
    }

    pub fn mesh_mut(&mut self) -> &mut TetMesh {
        &mut self.mesh
    }
}

impl UIElement for ElasticModel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Type du modèle élastique")
            .selected_text(format!("{:?}", self.model_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.model_type,
                    ElasticModelType::StVK,
                    format!("{:?}", ElasticModelType::StVK),
                );
                ui.selectable_value(
                    &mut self.model_type,
                    ElasticModelType::Corotationnal,
                    format!("{:?}", ElasticModelType::Corotationnal),
                );
            });
    }
}
