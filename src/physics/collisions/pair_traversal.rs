use super::{AABB, Body, TrianglePair};

pub struct PairTraversal<'a> {
    body_a: &'a Body,
    body_b: &'a Body,
    leaf_pairs: Vec<TrianglePair>,
    bounds_tests: usize,
}

impl<'a> PairTraversal<'a> {
    pub fn new(body_a: &'a Body, body_b: &'a Body) -> Self {
        Self {
            body_a,
            body_b,
            leaf_pairs: Vec::new(),
            bounds_tests: 0,
        }
    }

    pub fn run(&mut self) {
        self.descend(self.body_a.bounds().root(), self.body_b.bounds().root());
    }

    fn test_bounds(&mut self, a: &AABB, b: &AABB) -> bool {
        self.bounds_tests += 1;
        AABB::overlap(a, b)
    }

    fn descend(&mut self, node_a: usize, node_b: usize) {
        let transform_a = self.body_a.transform();
        let transform_a = transform_a.lock().unwrap();
        let transform_b = self.body_b.transform();
        let transform_b = transform_b.lock().unwrap();

        let aabb_node_a = &self.body_a.bounds().nodes()[node_a];
        let aabb_node_b = &self.body_b.bounds().nodes()[node_b];

        let aabb_a = aabb_node_a.bounds.transform(&transform_a);
        let aabb_b = aabb_node_b.bounds.transform(&transform_b);
        drop(transform_a);
        drop(transform_b);

        if !self.test_bounds(&aabb_a, &aabb_b) {
            return;
        } else if aabb_node_a.is_leaf() && aabb_node_b.is_leaf() {
            self.leaf_pairs.push(TrianglePair {
                a: aabb_node_a.ty.unwrap_leaf(),
                b: aabb_node_b.ty.unwrap_leaf(),
            });
            return;
        }

        if aabb_node_b.is_leaf()
            || (!aabb_node_a.is_leaf()
                && aabb_a.half_extents.norm_squared() > aabb_b.half_extents.norm_squared())
        {
            let node_a = aabb_node_a.ty.unwrap_node();
            self.descend(node_a.0, node_b);
            self.descend(node_a.1, node_b);
        } else {
            let node_b = aabb_node_b.ty.unwrap_node();
            self.descend(node_a, node_b.0);
            self.descend(node_a, node_b.1);
        }
    }

    pub fn leaf_pairs(&self) -> &[TrianglePair] {
        &self.leaf_pairs
    }

    pub fn bounds_tests(&self) -> usize {
        self.bounds_tests
    }
}
