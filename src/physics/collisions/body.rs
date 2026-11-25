use std::{rc::Rc, sync::Mutex};

use nalgebra::{Point3, Vector3, Vector4};

use crate::{
    physics::{
        collisions::{aabb::AABBTree, narrow_phase::Contact},
        tri_mesh::TriMesh,
    },
    renderer::{MeshRenderer, Renderable, Renderer},
    transform::Transform,
};

pub struct BodyContact {
    pub vertex_body: usize,
    pub vertex: usize,
    pub triangle_body: usize,
    pub triangle: usize,
    pub contact: Contact,
}

pub struct Body {
    mesh: TriMesh,
    mesh_renderer: MeshRenderer,
    transform: Rc<Mutex<Transform<f64>>>,
    velocity: Vector3<f64>,
    angular_velocity: Vector3<f64>,
    bounds: AABBTree,
    pub show_bounds: bool,
}

impl Body {
    pub fn new(
        mesh: TriMesh,
        transform: Transform<f64>,
        velocity: Vector3<f64>,
        angular_velocity: Vector3<f64>,
        renderer: &Renderer,
    ) -> Self {
        let mesh_renderer = MeshRenderer::new(renderer, mesh.vertices(), &mesh.indices());
        mesh_renderer.update_transform_buffer(&transform, renderer);
        let transform = Rc::new(Mutex::new(transform));
        let bounds = AABBTree::build(transform.clone(), &mesh, 0.02, renderer);

        Self {
            mesh,
            mesh_renderer,
            transform,
            velocity,
            angular_velocity,
            bounds,
            show_bounds: false,
        }
    }

    pub fn step(&mut self, h: f64) {
        let angle = h * self.angular_velocity.norm();
        let mut transform = self.transform.lock().unwrap();

        transform.translate(h * self.velocity);

        if angle > 0.0 {
            transform.rotate(self.angular_velocity, angle);
        }
    }

    pub fn update_mesh(&mut self, renderer: &Renderer) {
        self.mesh_renderer
            .update_transform_buffer(&self.transform.lock().unwrap(), renderer);
        self.bounds.update_mesh(renderer);
    }

    pub fn velocity(&self) -> &Vector3<f64> {
        &self.velocity
    }

    pub fn position(&self) -> Point3<f64> {
        let transform = self.transform.lock().unwrap();
        transform.position()
    }

    pub fn world_vertices(&self) -> Vec<Vector3<f64>> {
        let transform = self.transform.lock().unwrap();
        let matrix = transform.matrix();
        self.mesh
            .vertices()
            .iter()
            .map(|vertex| (matrix * Vector4::new(vertex.x, vertex.y, vertex.z, 1.0)).xyz())
            .collect()
    }

    pub fn radius(&self) -> f64 {
        1.0
    }

    pub fn set_position(&mut self, position: Point3<f64>) {
        let mut transform = self.transform.lock().unwrap();
        transform.set_translation(position.coords);
    }

    pub fn set_velocity(&mut self, velocity: Vector3<f64>) {
        self.velocity = velocity;
    }

    pub fn mesh(&self) -> &TriMesh {
        &self.mesh
    }

    pub fn bounds(&self) -> &AABBTree {
        &self.bounds
    }

    pub fn bounds_mut(&mut self) -> &mut AABBTree {
        &mut self.bounds
    }

    pub fn transform(&self) -> Rc<Mutex<Transform<f64>>> {
        self.transform.clone()
    }
}

impl Renderable for Body {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        self.mesh_renderer.render(renderer, render_pass);

        if self.show_bounds {
            self.bounds.render(renderer, render_pass);
        }
    }
}
