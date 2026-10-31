use std::ops::{AddAssign, SubAssign};

use nalgebra::{DVector, Matrix3, RowVector3, SMatrix, SVector, Vector3};
use nalgebra_sparse::{CooMatrix, CsrMatrix};

use super::{GRAVITY, stvk, tet_mesh::TetMesh};

pub struct LameParameters {
    pub mu: f64,
    pub lambda: f64,
}

impl LameParameters {
    pub fn new(mu: f64, lambda: f64) -> Self {
        Self { mu, lambda }
    }
}

pub struct ElasticModel {
    mesh: TetMesh,
    lame: LameParameters,
    lumped_masses: DVector<f64>,
    total_mass: f64,
    rayleigh_alpha: f64,
    rayleigh_beta: f64,
}

impl ElasticModel {
    pub fn new(mesh: TetMesh, lame: LameParameters, density: f64) -> Self {
        let (lumped_masses, total_mass) = Self::build_masses(&mesh, density);

        Self {
            mesh,
            lame,
            lumped_masses,
            total_mass,
            rayleigh_alpha: 0.0,
            rayleigh_beta: 0.0,
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

            energy += el.rest_volume() * stvk::energy_density(&F, &self.lame);
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
        let P = stvk::first_piola_kirchhoff(&F, &self.lame);

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

    pub fn compute_forces(&self, x: &DVector<f64>, v: &DVector<f64>) -> DVector<f64> {
        if self.rayleigh_beta != 0.0 {
            self.compute_forces_with_stiffness(x, v, &self.stiffness_matrix(x))
        } else {
            self.compute_forces_with_stiffness(
                x,
                v,
                &CsrMatrix::from(&CooMatrix::new(
                    self.mesh.dof_count(),
                    self.mesh.dof_count(),
                )),
            )
        }
    }

    pub fn compute_forces_with_stiffness(
        &self,
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

    fn element_stiffness(&self, element_idx: usize, x: &DVector<f64>) -> SMatrix<f64, 12, 12> {
        let tet = &self.mesh.tets()[element_idx];
        let element = &self.mesh.elements()[element_idx];

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
                let delta_P = stvk::delta_first_piola_kirchhoff(&F, &delta_F, &self.lame);
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

    pub fn stiffness_matrix(&self, x: &DVector<f64>) -> CsrMatrix<f64> {
        let mut matrix = CooMatrix::<f64>::new(self.mesh.dof_count(), self.mesh.dof_count());

        for e in 0..self.mesh.tet_count() {
            let tet = &self.mesh.tets()[e];
            let local = self.element_stiffness(e, x);

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

    pub fn damping_matrix(&self, stiffness: &CsrMatrix<f64>) -> CsrMatrix<f64> {
        let n = self.lumped_masses.len();
        let mut matrix = CooMatrix::new(n, n);

        if self.rayleigh_alpha != 0.0 {
            for (i, val) in self.lumped_masses.iter().enumerate() {
                matrix.push(i, i, self.rayleigh_alpha * val);
            }
        }

        if self.rayleigh_beta != 0.0 {
            for (i, j, val) in stiffness.triplet_iter() {
                matrix.push(i, j, self.rayleigh_beta * val);
            }
        }

        CsrMatrix::from(&matrix)
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
}
