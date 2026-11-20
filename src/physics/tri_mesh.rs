use nalgebra::Vector3;

pub struct TriMesh {
    vertices: Vec<Vector3<f64>>,
    triangles: Vec<[u32; 3]>,
}

impl TriMesh {
    pub fn load_obj(path: &str) -> Self {
        let obj = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .expect("Impossible de charger le fichier OBJ.");

        let (model, _) = obj;
        let model = &model[0];

        let vertices = model
            .mesh
            .positions
            .chunks_exact(3)
            .map(|v| Vector3::new(v[0] as f64, v[1] as f64, v[2] as f64))
            .collect();

        let triangles = model
            .mesh
            .indices
            .chunks_exact(3)
            .map(|i| [i[0], i[1], i[2]])
            .collect();

        Self {
            vertices,
            triangles,
        }
    }

    pub fn triangle_neighbours(&self) -> Vec<Vec<usize>> {
        let mut edges: Vec<[u32; 3]> = Vec::with_capacity(self.triangle_count());

        for triangle in 0..self.triangle_count() {
            for k in 0..3 {
                let u = self.triangles[triangle][k];
                let v = self.triangles[triangle][(k + 1) % 3];
                edges.push([u.min(v), u.max(v), triangle as u32]);
            }
        }
        edges.sort();

        let mut neighbours = vec![Vec::new(); self.triangle_count()];

        let mut first = 0;
        while first < edges.len() {
            let mut last = first + 1;
            while last < edges.len()
                && edges[last][0] == edges[first][0]
                && edges[last][1] == edges[first][1]
            {
                last += 1;
            }

            for i in first..last {
                for j in (i + 1)..last {
                    if edges[i][2] != edges[j][2] {
                        neighbours[edges[i][2] as usize].push(edges[j][2] as usize);
                        neighbours[edges[j][2] as usize].push(edges[i][2] as usize);
                    }
                }
            }

            first = last;
        }

        for list in &mut neighbours {
            list.sort();
            list.dedup();
        }

        neighbours
    }

    pub fn vertices(&self) -> &[Vector3<f64>] {
        &self.vertices
    }

    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }

    pub fn indices(&self) -> Vec<u32> {
        self.triangles.iter().map(|&i| i).flatten().collect()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }
}
