use std::{fmt::Debug, sync::Arc};

use parking_lot::RwLock;

use crate::rendering::{material::Material, mesh::Mesh};

#[derive(Clone, Debug)]
pub struct MeshRenderer {
    pub mesh: Arc<RwLock<Mesh>>,
    pub material: Arc<RwLock<dyn Material>>,
    pub render_layer: String,
}
