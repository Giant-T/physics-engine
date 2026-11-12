use std::{collections::HashMap, fs::File, io::Read};

use nalgebra::{Matrix3, Vector3};

const K_PERMUTATIONS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

const K_TET_FACES: [[usize; 3]; 4] = [[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]];

pub struct TetElement {
    rest_inverse: Matrix3<f64>,
    rest_volume: f64,
}

impl TetElement {
    pub fn new(
        x0: &Vector3<f64>,
        x1: &Vector3<f64>,
        x2: &Vector3<f64>,
        x3: &Vector3<f64>,
    ) -> Option<Self> {
        let rest = Self::shape_matrix(x0, x1, x2, x3);

        if rest.determinant() == 0.0 {
            return None;
        }

        let rest_inverse = rest.try_inverse().unwrap();
        let rest_volume = rest.determinant().abs() / 6.0;

        Some(Self {
            rest_inverse,
            rest_volume,
        })
    }

    pub fn shape_matrix(
        p0: &Vector3<f64>,
        p1: &Vector3<f64>,
        p2: &Vector3<f64>,
        p3: &Vector3<f64>,
    ) -> Matrix3<f64> {
        Matrix3::from_columns(&[p1 - p0, p2 - p0, p3 - p0])
    }

    pub fn rest_volume(&self) -> f64 {
        self.rest_volume
    }

    pub fn rest_inverse(&self) -> &Matrix3<f64> {
        &self.rest_inverse
    }

    pub fn deformation_gradient(
        &self,
        x0: &Vector3<f64>,
        x1: &Vector3<f64>,
        x2: &Vector3<f64>,
        x3: &Vector3<f64>,
    ) -> Matrix3<f64> {
        Self::shape_matrix(x0, x1, x2, x3) * self.rest_inverse
    }
}

pub type Tet = [usize; 4];

pub struct TetMesh {
    rest_positions: Vec<Vector3<f64>>,
    tets: Vec<Tet>,
    pinned: Vec<bool>,
    elements: Vec<TetElement>,
}

impl TetMesh {
    fn ensure_positive_orientation(positions: &Vec<Vector3<f64>>, tet: &mut Tet) {
        let shape = Matrix3::from_columns(&[
            positions[tet[1]] - positions[tet[0]],
            positions[tet[2]] - positions[tet[0]],
            positions[tet[3]] - positions[tet[0]],
        ]);

        if shape.determinant() < 0.0 {
            tet.swap(2, 3);
        }
    }

    fn build_elements(positions: &Vec<Vector3<f64>>, tets: &Vec<Tet>) -> Vec<TetElement> {
        let mut elements = Vec::with_capacity(tets.len());

        for tet in tets {
            elements.push(
                TetElement::new(
                    &positions[tet[0]],
                    &positions[tet[1]],
                    &positions[tet[2]],
                    &positions[tet[3]],
                )
                .unwrap(),
            );
        }

        elements
    }

    pub fn load_mesh_file(path: &str) -> Self {
        log::info!("TetMesh::load_mesh_file");

        let mut buf = String::new();
        File::open(path).unwrap().read_to_string(&mut buf).unwrap();
        let mut lines = buf.lines().map(|line| line.trim());

        if let None = lines.find(|&line| line == "Vertices") {
            panic!("Section Vertices absente de {path}");
        };
        let node_count = usize::from_str_radix(lines.next().unwrap(), 10).unwrap();
        if node_count == 0 {
            panic!("Nombre de sommet invalide dans {path}");
        }
        let mut rest_positions = Vec::with_capacity(node_count);

        for line in lines.by_ref().take(node_count) {
            let mut nums = line.split(' ');
            let mut coords = nums.by_ref().take(3).map(|n| n.parse::<f64>().unwrap());

            rest_positions.push(Vector3::new(
                coords.next().unwrap(),
                coords.next().unwrap(),
                coords.next().unwrap(),
            ));
        }

        if let None = lines.find(|line| *line == "Tetrahedra") {
            panic!("Section Tetrahedra absente de {path}");
        };

        let tet_count = usize::from_str_radix(lines.next().unwrap(), 10).unwrap();
        if tet_count == 0 {
            panic!("Nombre de tetraedre invalide dans {path}");
        }
        let mut tets = Vec::with_capacity(tet_count);

        for line in lines.take(tet_count) {
            let position_indices = line
                .split(' ')
                .take(4)
                .map(|n| usize::from_str_radix(n, 10).unwrap());

            let mut tet: Tet = [0; 4];

            for (i, position_idx) in position_indices.enumerate() {
                let position_idx = position_idx - 1;
                if position_idx >= node_count {
                    panic!("Indice de noeud {position_idx} hors bornes {node_count} dans {path}");
                }
                tet[i] = position_idx;
            }

            Self::ensure_positive_orientation(&rest_positions, &mut tet);
            tets.push(tet);
        }

        let elements = Self::build_elements(&rest_positions, &tets);

        let pinned = vec![false; node_count];

        Self {
            rest_positions,
            tets,
            pinned,
            elements,
        }
    }

    pub fn box_mesh(cells_x: usize, cells_y: usize, cells_z: usize, cell_size: f64) -> Self {
        log::info!("TetMesh::box_mesh");

        if cells_x == 0 || cells_y == 0 || cells_z == 0 {
            panic!("Il doit y avoir au moins une cellule par axe.");
        }

        let mut rest_positions = Vec::new();

        for k in 0..=cells_z {
            for j in 0..=cells_y {
                for i in 0..=cells_x {
                    rest_positions.push(Vector3::new(
                        i as f64 * cell_size,
                        j as f64 * cell_size,
                        k as f64 * cell_size,
                    ));
                }
            }
        }

        let nx = cells_x + 1;
        let ny = cells_y + 1;
        let node_index = |i: usize, j: usize, k: usize| i + nx * (j + ny * k);

        let mut tets = Vec::new();
        for cz in 0..cells_z {
            for cy in 0..cells_y {
                for cx in 0..cells_x {
                    let corner = |dx: usize, dy: usize, dz: usize| {
                        return node_index(cx + dx, cy + dy, cz + dz);
                    };

                    for permutation in K_PERMUTATIONS {
                        let mut offset = [0; 3];
                        let mut tet: Tet = [0; 4];
                        tet[0] = corner(0, 0, 0);

                        for step in 0..3 {
                            offset[permutation[step]] = 1;
                            tet[step + 1] = corner(offset[0], offset[1], offset[2]);
                        }

                        Self::ensure_positive_orientation(&rest_positions, &mut tet);
                        tets.push(tet);
                    }
                }
            }
        }

        let elements = Self::build_elements(&rest_positions, &tets);
        let pinned = vec![false; rest_positions.len()];

        Self {
            rest_positions,
            tets,
            elements,
            pinned,
        }
    }

    pub fn surface(&self) -> Vec<[u32; 3]> {
        let mut faces_count = HashMap::new();

        for tet in &self.tets {
            for face in K_TET_FACES {
                let face = [tet[face[0]], tet[face[1]], tet[face[2]]];
                let mut key = face.clone();
                key.sort();

                faces_count
                    .entry(key)
                    .and_modify(|(counter, _)| *counter += 1)
                    .or_insert((1, face));
            }
        }

        faces_count
            .values()
            .filter_map(|(count, face)| {
                if *count == 1 {
                    Some([face[0] as u32, face[1] as u32, face[2] as u32])
                } else {
                    None
                }
            })
            .collect()
    }

    fn bounding_box_min(&self) -> Vector3<f64> {
        self.rest_positions
            .iter()
            .fold(self.rest_positions.first().unwrap().clone(), |acc, el| {
                acc.inf(el)
            })
    }

    fn bounding_box_max(&self) -> Vector3<f64> {
        self.rest_positions
            .iter()
            .fold(self.rest_positions.first().unwrap().clone(), |acc, el| {
                acc.sup(el)
            })
    }

    pub fn pin_left_face(&mut self, tolerance: f64) -> usize {
        let left = self.bounding_box_min().x;

        let mut count = 0;
        for i in 0..self.node_count() {
            if self.rest_positions[i].x <= left + tolerance {
                self.pinned[i] = true;
                count += 1;
            }
        }
        count
    }

    pub fn pin_above(&mut self, height_fraction: f64) -> usize {
        let low = self.bounding_box_min().y;
        let high = self.bounding_box_max().y;
        let threshold = low + height_fraction * (high - low);

        let mut count = 0;
        for i in 0..self.node_count() {
            if self.rest_positions[i].y >= threshold {
                self.pinned[i] = true;
                count += 1;
            }
        }
        count
    }

    pub fn pin_below(&mut self, height_fraction: f64) -> usize {
        let low = self.bounding_box_min().y;
        let high = self.bounding_box_max().y;
        let threshold = low + height_fraction * (high - low);

        let mut count = 0;
        for i in 0..self.node_count() {
            if self.rest_positions[i].y <= threshold {
                self.pinned[i] = true;
                count += 1;
            }
        }
        count
    }

    pub fn elements(&self) -> &[TetElement] {
        &self.elements
    }

    pub fn pinned(&self) -> &[bool] {
        &self.pinned
    }

    pub fn pinned_count(&self) -> usize {
        self.pinned.iter().filter(|p| **p).count()
    }

    pub fn tets(&self) -> &[Tet] {
        &self.tets
    }

    pub fn pins(&self) -> Vec<Vector3<f64>> {
        self.rest_positions
            .iter()
            .enumerate()
            .filter_map(|(i, pos)| {
                if self.pinned[i] {
                    Some(pos.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn node_count(&self) -> usize {
        self.rest_positions.len()
    }

    pub fn dof_count(&self) -> usize {
        3 * self.node_count()
    }

    pub fn tet_count(&self) -> usize {
        self.tets.len()
    }

    pub fn is_pinned(&self, index: usize) -> bool {
        self.pinned[index]
    }

    pub fn rest_positions(&self) -> &[Vector3<f64>] {
        &self.rest_positions
    }
}
