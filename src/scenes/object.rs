use crate::{
    physics::{elastic_model::ElasticModel, tet_mesh::TetMesh},
    renderer::{MeshRenderer, Renderable, Renderer},
};

pub struct SceneObject {
    name: &'static str,
    // elastic_model: ElasticModel,
    mesh_renderer: MeshRenderer,
}

impl SceneObject {
    pub fn new(mesh_path: &'static str, renderer: &Renderer) -> Self {
        let tet_mesh = TetMesh::load_mesh_file(mesh_path);
        let mesh_renderer = MeshRenderer::from_tet_mesh(&tet_mesh, renderer);

        // let elastic_model = ElasticModel::new(tet_mesh, lame, density);

        Self {
            name: mesh_path.split('/').last().unwrap(),
            // elastic_model,
            mesh_renderer,
        }
    }
}

impl Renderable for SceneObject {
    fn render(&self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass) {
        self.mesh_renderer.render(renderer, render_pass);
    }
}
