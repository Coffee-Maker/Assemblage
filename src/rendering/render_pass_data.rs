use std::{collections::HashMap, sync::Arc};

use crate::{
    asset_types::{asset::Asset, mesh::Mesh},
    state::State,
};

use super::{material::Material, vertex::Vertex};
use flume::Receiver;
use glam::{Mat4, Vec3};
use parking_lot::RwLock;
use wgpu::util::DeviceExt;

// Render layers are a convenient way to filter what a camera renders
// They also make for a convenient location to store render passes
pub mod render_layers {
    use super::{create_render_pass, RenderPassData};
    use crate::{rendering::material::Material, state::State};
    use dashmap::DashMap;
    use parking_lot::RwLock;
    use std::{collections::HashMap, sync::Arc};

    lazy_static! {
        pub static ref RENDER_LAYERS: DashMap<String, Arc<RwLock<RenderLayer>>> =
            DashMap::default();
    }

    #[derive(Debug)]
    pub struct RenderLayer {
        pub name: String,
        pub passes: HashMap<u64, Arc<RwLock<RenderPassData<dyn Material>>>>,
    }

    impl RenderLayer {
        pub fn new(name: String) -> Self {
            Self {
                name,
                passes: HashMap::new(),
            }
        }

        pub fn add_pass(&mut self, pass: Arc<RwLock<RenderPassData<dyn Material>>>) {
            self.passes.insert(pass.read().id, Arc::clone(&pass));
        }

        pub fn remove_pass(&mut self, pass_id: u64) {
            self.passes.remove(&pass_id);
        }

        pub fn get_or_create_pass(
            &mut self,
            state: &State,
            material: Arc<RwLock<dyn Material>>,
        ) -> Arc<RwLock<RenderPassData<dyn Material>>> {
            let id = material.read().get_id();
            if self.passes.contains_key(&id) {
                Arc::clone(self.passes.get(&id).unwrap())
            } else {
                self.passes.insert(
                    id,
                    Arc::new(RwLock::new(create_render_pass(
                        state,
                        Arc::clone(&material),
                        id,
                    ))),
                );
                Arc::clone(self.passes.get(&id).unwrap())
            }
        }
    }

    pub fn get_layer_by_name(name: String) -> Option<Arc<RwLock<RenderLayer>>> {
        RENDER_LAYERS
            .get(&name)
            .map(|layer| Arc::clone(layer.value()))
    }

    pub fn create_layer(name: String) {
        RENDER_LAYERS.insert(name.clone(), Arc::new(RwLock::new(RenderLayer::new(name))));
    }
}

#[derive(Debug)]
pub struct RenderMesh {
    pub mesh: Arc<RwLock<Mesh>>,
    pub transform: Mat4,
}

#[derive(Debug)]
pub struct RenderPassData<M: Material + ?Sized> {
    pub material: Arc<RwLock<M>>,
    pub meshes: HashMap<u64, RenderMesh>,
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub id: u64,
    pub dirty: bool,
}

impl RenderPassData<dyn Material> {
    fn set_vertices(&mut self, device: &wgpu::Device, vertices: Vec<Vertex>) {
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.vertex_count = vertices.len() as u32;
    }

    fn set_indices(&mut self, device: &wgpu::Device, indices: Vec<u32>) {
        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.index_count = indices.len() as u32;
    }

    pub fn insert_mesh(&mut self, mesh: RenderMesh) {
        let id = mesh.mesh.read().get_id();
        self.meshes.insert(id, mesh);
        self.dirty = true;
    }

    pub fn update_buffers(&mut self, device: &wgpu::Device) {
        let mut combined_verts = Vec::new();
        let mut combined_indices = Vec::new();
        self.meshes.iter().for_each(|(_id, mesh)| {
            let offset = combined_verts.len() as u32;
            let mesh_lock = mesh.mesh.read();
            combined_verts.reserve(mesh_lock.vertex_count);
            combined_verts.append(
                &mut mesh_lock
                    .get_vertices()
                    .iter()
                    .map(|vertex| {
                        let mut vert = vertex.clone();
                        let vert_pos = mesh.transform.transform_point3(Vec3::from(vert.position));
                        vert.position = vert_pos.to_array();
                        vert
                    })
                    .collect(),
            );

            combined_indices.extend(mesh_lock.get_indices().iter().map(|&x| x + offset));
        });
        self.set_indices(device, combined_indices);
        self.set_vertices(device, combined_verts);

        self.dirty = false;
    }
}

pub fn create_render_pass(
    state: &State,
    material: Arc<RwLock<dyn Material>>,
    pass_id: u64,
) -> RenderPassData<dyn Material> {
    let vertex_buffer = state
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX,
        });

    let index_buffer = state
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::INDEX,
        });

    let vertex_count = 0;
    let index_count = 0;

    let meshes = HashMap::new();

    RenderPassData {
        material: Arc::clone(&material),
        meshes,
        vertex_buffer,
        index_buffer,
        vertex_count,
        index_count,
        id: pass_id,
        dirty: true,
    }
}
