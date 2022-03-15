use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

use parking_lot::RwLock;

use crate::{
    asset_types::{asset::Asset, mesh::Mesh},
    next_id,
    rendering::material::Material,
};

#[derive(Clone)]
pub struct MeshRenderer {
    pub mesh: Arc<RwLock<Mesh>>,
    pub material: Arc<RwLock<dyn Material>>,
    pub render_layer: String,
    pub dirty: Arc<AtomicBool>,
    id: u64,
}

impl MeshRenderer {
    pub fn new(
        mesh: Arc<RwLock<Mesh>>,
        material: Arc<RwLock<dyn Material>>,
        render_layer: String,
    ) -> Self {
        let mut r = Self {
            mesh,
            material,
            render_layer,
            dirty: Arc::new(AtomicBool::new(true)),
            id: next_id(),
        };
        r.listen_for_changes();
        r
    }

    fn listen_for_changes(&mut self) {
        let mut change_listener = self.mesh.write().get_change_receiver();
        let dirty_clone = Arc::clone(&self.dirty);
        rayon::spawn(move || {
            change_listener.recv().unwrap();
            dirty_clone.store(true, std::sync::atomic::Ordering::Relaxed);
        });
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
}
