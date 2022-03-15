use std::sync::Arc;

use crate::{asset_types::mesh::Mesh, state::State};

use wgpu::{BufferDescriptor, BufferUsages};

use super::material::Material;
use glam::Mat4;
use parking_lot::RwLock;

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
pub struct MeshBufferEntry {
    pub vertex_start: usize,
    pub vertex_length: usize,
    pub index_start: usize,
    pub index_length: usize,
}

#[derive(Debug)]
pub struct MeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub vertex_offset: u64,
    pub index_offset: u64,
    pub vertex_count: u32,
    pub index_count: u32,
}

impl MeshBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: 500_000_000, // 500mb (Maybe too much!)
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Index Buffer"),
            size: 500_000_000, // 500mb (Maybe too much!)
            usage: BufferUsages::COPY_DST | BufferUsages::INDEX,
            mapped_at_creation: false,
        });
        MeshBuffer {
            vertex_buffer,
            index_buffer,
            vertex_offset: 0,
            index_offset: 0,
            vertex_count: 0,
            index_count: 0,
        }
    }

    pub fn insert_mesh(&mut self, state: &State, mesh: Arc<RwLock<Mesh>>, transform: &Mat4) {
        let mesh_lock = mesh.read();
        // Prepare data
        let mut new_vertices = mesh_lock.get_vertices().clone();
        new_vertices.iter_mut().for_each(|vertex| {
            vertex.position = transform.transform_point3(vertex.position.into()).into();
            // Transforming the normal is not always required, perhaps find a way to avoid doing this in those cases
            vertex.normal = transform.transform_vector3(vertex.normal.into()).into();
        });
        let vertex_data = bytemuck::cast_slice(&new_vertices);

        let mut new_indices = mesh_lock.get_indices().clone();
        new_indices
            .iter_mut()
            .for_each(|index| *index += self.vertex_count);
        let index_data = bytemuck::cast_slice(&new_indices);

        // write data into buffers
        state
            .queue
            .write_buffer(&self.vertex_buffer, self.vertex_offset, vertex_data);
        state
            .queue
            .write_buffer(&self.index_buffer, self.index_offset, index_data);

        self.vertex_offset += vertex_data.len() as u64;
        self.index_offset += index_data.len() as u64;
        self.vertex_count += mesh_lock.vertex_count as u32;
        self.index_count += mesh_lock.index_count as u32;
    }
}

#[derive(Debug)]
pub struct RenderPassData<M: Material + ?Sized> {
    pub material: Arc<RwLock<M>>,
    pub buffer: MeshBuffer,
    pub id: u64,
}

impl RenderPassData<dyn Material> {
    pub fn insert_mesh(&mut self, state: &State, mesh: Arc<RwLock<Mesh>>, transform: &Mat4) {
        self.buffer.insert_mesh(state, mesh, transform)
    }
}

pub fn create_render_pass(
    state: &State,
    material: Arc<RwLock<dyn Material>>,
    pass_id: u64,
) -> RenderPassData<dyn Material> {
    RenderPassData {
        material: Arc::clone(&material),
        id: pass_id,
        buffer: MeshBuffer::new(&state.device),
    }
}
