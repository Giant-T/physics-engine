use super::{Anchoring, ElasticMeshType};

pub enum SceneType {
    Elastic {
        mesh_type: ElasticMeshType,
        anchoring: Anchoring,
    },
    CollisionDetections {
        n_bodies: usize,
        half_size: f64,
    },
}

pub struct SceneDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub time_step: f64,
    pub scene_type: SceneType,
}
