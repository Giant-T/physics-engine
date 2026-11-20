use std::{rc::Rc, sync::Mutex};

use nalgebra::{Point3, Vector3};

use crate::{
    physics::tri_mesh::TriMesh,
    renderer::{AABBRenderer, Renderable, Renderer},
    transform::Transform,
};

#[derive(Clone)]
pub struct AABB {
    pub center: Point3<f64>,
    pub half_extents: Vector3<f64>,
}

impl AABB {
    pub fn from_triangle(mesh: &TriMesh, triangle: usize, margin: f64) -> Self {
        let a = mesh.vertices()[mesh.triangles()[triangle][0] as usize];
        let b = mesh.vertices()[mesh.triangles()[triangle][1] as usize];
        let c = mesh.vertices()[mesh.triangles()[triangle][2] as usize];

        let max = a.sup(&b).sup(&c) + Vector3::repeat(margin / 2.0);
        let min = a.inf(&b).inf(&c) - Vector3::repeat(margin / 2.0);

        Self {
            center: ((min + max) * 0.5).into(),
            half_extents: (max - min) * 0.5,
        }
    }

    fn min(&self) -> Vector3<f64> {
        self.center.coords - self.half_extents
    }

    fn max(&self) -> Vector3<f64> {
        self.center.coords + self.half_extents
    }

    pub fn squared_dist(&self, other: &Self) -> f64 {
        (self.center - other.center).norm_squared()
    }

    pub fn transform(&self, transform: &Transform<f64>) -> Self {
        Self {
            center: transform.point_to_world(&self.center),
            half_extents: transform.vector_to_world(&self.half_extents),
        }
    }

    pub fn overlap(a: &Self, b: &Self) -> bool {
        let overlap = a.half_extents + b.half_extents - (a.center - b.center).abs();

        overlap.min() >= 0.0
    }

    pub fn fuse(aabb1: &Self, aabb2: &Self) -> Self {
        let max = aabb1.max().sup(&aabb2.max());
        let min = aabb1.min().inf(&aabb2.min());

        Self {
            center: ((min + max) * 0.5).into(),
            half_extents: (max - min) * 0.5,
        }
    }
}

pub enum AABBBNodeType {
    Node { left: usize, right: usize },
    Leaf { triangle: usize },
}

impl AABBBNodeType {
    pub fn unwrap_node(&self) -> (usize, usize) {
        if let Self::Node { left, right } = self {
            (*left, *right)
        } else {
            panic!()
        }
    }

    pub fn unwrap_leaf(&self) -> usize {
        if let Self::Leaf { triangle } = self {
            *triangle
        } else {
            panic!()
        }
    }
}

pub struct AABBNode {
    pub bounds: AABB,
    pub ty: AABBBNodeType,
}

impl AABBNode {
    pub fn is_leaf(&self) -> bool {
        if let AABBBNodeType::Leaf { .. } = self.ty {
            true
        } else {
            false
        }
    }
}

pub struct AABBTree {
    transform: Rc<Mutex<Transform<f64>>>,
    nodes: Vec<AABBNode>,
    root: usize,
    margin: f64,
    bounds_renderer: AABBRenderer,
    triangle_count: usize,
}

impl AABBTree {
    pub fn build(
        transform: Rc<Mutex<Transform<f64>>>,
        mesh: &TriMesh,
        margin: f64,
        renderer: &Renderer,
    ) -> Self {
        let triangle_count = mesh.triangle_count();

        let mut nodes = Vec::with_capacity(2 * triangle_count - 1);
        let mut level = Vec::with_capacity(triangle_count);
        for triangle in 0..triangle_count {
            let leaf = AABBNode {
                bounds: AABB::from_triangle(mesh, triangle, margin),
                ty: AABBBNodeType::Leaf { triangle },
            };
            nodes.push(leaf);
            level.push(triangle);
        }
        let mut neighbours = mesh.triangle_neighbours();
        Self::connect_isolated(&level, &mut neighbours);

        while level.len() > 1 {
            let next = Self::merge_level(&level, &neighbours, &mut nodes);

            assert!(!next.is_empty(), "merge_level retourne un niveau vide");
            assert!(
                next.len() < level.len(),
                "un niveau ne fusionne aucun noeud"
            );

            neighbours = Self::next_neighbours(&level, &next, &nodes, &neighbours);
            level = next;
        }

        let transform_val = transform.lock().unwrap();
        let aabbs: Vec<AABB> = nodes
            .iter()
            .map(|node| node.bounds.transform(&transform_val))
            .collect();
        let bounds_renderer = AABBRenderer::from_aabb_list(&aabbs, renderer);

        Self {
            transform: transform.clone(),
            nodes,
            root: level[0],
            margin,
            bounds_renderer,
            triangle_count,
        }
    }

    fn next_neighbours(
        level: &[usize],
        next: &[usize],
        nodes: &[AABBNode],
        neighbours: &[Vec<usize>],
    ) -> Vec<Vec<usize>> {
        let count = nodes.len();
        let mut in_level = vec![false; count];
        for &node in level {
            in_level[node] = true;
        }

        let mut owner = vec![None; count];
        for &node in next {
            assert!(
                node < count,
                "merge_level retourne un indice de noeud qui n'existe pas"
            );
            if in_level[node] {
                owner[node] = Some(node);
                continue;
            }

            let parent = &nodes[node];
            if let AABBBNodeType::Node { left, right } = parent.ty {
                assert!(
                    left < count && right < count && in_level[left] && in_level[right],
                    "un noeud parent du niveau suivant n'a pas ses deux noeuds dans le niveau."
                );
                owner[left] = Some(node);
                owner[right] = Some(node);
            }
        }

        let mut result = vec![Vec::new(); count];
        for &node in next {
            let current = &nodes[node];
            let AABBBNodeType::Node { left, right } = current.ty else {
                continue;
            };

            let children = if in_level[node] {
                vec![node]
            } else {
                vec![left, right]
            };

            let list = &mut result[node];
            for &child in &children {
                for &other in &neighbours[child] {
                    if let Some(other_owner) = owner[other] {
                        if other_owner != node {
                            list.push(other_owner);
                        }
                    }
                }

                list.sort();
                list.dedup();
            }
        }

        Self::connect_isolated(next, &mut result);
        result
    }

    fn merge_level(
        level: &[usize],
        neighbours: &[Vec<usize>],
        nodes: &mut Vec<AABBNode>,
    ) -> Vec<usize> {
        let mut next_nodes = Vec::new();
        let mut free_nodes = level.to_vec();

        for &node_idx in level {
            if !free_nodes.contains(&node_idx) {
                continue;
            }

            let node = &nodes[node_idx];
            let mut closest_node_idx = neighbours[node_idx][0];
            let mut closest_dist = node.bounds.squared_dist(&nodes[closest_node_idx].bounds);

            for &neighbour_idx in &neighbours[node_idx] {
                if !free_nodes.contains(&neighbour_idx) {
                    continue;
                }

                let neighbour = &nodes[neighbour_idx];
                let neighbour_dist = node.bounds.squared_dist(&neighbour.bounds);

                if neighbour_dist < closest_dist {
                    closest_node_idx = neighbour_idx;
                    closest_dist = neighbour_dist;
                }
            }

            if !free_nodes.contains(&closest_node_idx) {
                continue;
            }

            nodes.push(AABBNode {
                bounds: AABB::fuse(&node.bounds, &nodes[closest_node_idx].bounds),
                ty: AABBBNodeType::Node {
                    left: node_idx,
                    right: closest_node_idx,
                },
            });
            next_nodes.push(nodes.len() - 1);
            free_nodes.retain(|&free_node| free_node != node_idx && free_node != closest_node_idx);
        }

        next_nodes.extend(free_nodes);

        next_nodes
    }

    fn connect_isolated(level: &[usize], neighbours: &mut [Vec<usize>]) {
        if level.len() < 2 {
            return;
        }

        let mut isolated = vec![false; neighbours.len()];
        let mut any = false;
        for node in level {
            if neighbours[*node].is_empty() {
                isolated[*node] = true;
                any = true;
            }
        }

        if !any {
            return;
        }

        for node in level {
            for other in level {
                if other != node && (isolated[*node] || isolated[*other]) {
                    neighbours[*node].push(*other);
                }
            }
        }
    }

    pub fn update_mesh(&mut self, renderer: &Renderer) {
        let aabbs: Vec<AABB> = self
            .nodes
            .iter()
            .map(|node| node.bounds.transform(&self.transform.lock().unwrap()))
            .collect();
        self.bounds_renderer.set_aabbs(&aabbs, renderer);
    }

    pub fn nodes(&self) -> &[AABBNode] {
        &self.nodes
    }

    pub fn root(&self) -> usize {
        self.root
    }
}

impl Renderable for AABBTree {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        self.bounds_renderer.render(renderer, render_pass);
    }
}
