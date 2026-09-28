use super::Renderer;
use super::mesh::Mesh;
use super::transform::Transform;

pub struct RenderObject {
    transform: Transform,
    mesh: Mesh,
}
impl RenderObject {
    pub fn new(transform: Transform, mesh: Mesh) -> Self {
        Self { transform, mesh }
    }

    pub fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        self.mesh.render(renderer, render_pass);
    }
}
