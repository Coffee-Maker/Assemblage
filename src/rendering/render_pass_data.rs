use std::sync::Arc;

use crate::state::State;

use super::{material::Material, vertex::Vertex};
use parking_lot::RwLock;
use wgpu::util::DeviceExt;

// Render layers are a convenient way to filter what a camera renders
// They also make for a convenient location to store render passes
pub mod render_layers {
    use super::RenderPassData;
    use crate::rendering::material::Material;
    use dashmap::DashMap;
    use parking_lot::RwLock;
    use std::sync::Arc;

    lazy_static! {
        pub static ref RENDER_LAYERS: DashMap<String, Arc<RwLock<RenderLayer>>> =
            DashMap::default();
    }

    #[derive(Debug)]
    pub struct RenderLayer {
        pub name: String,
        pub passes: Vec<Arc<RwLock<RenderPassData<dyn Material>>>>,
    }

    impl RenderLayer {
        pub fn new(name: String) -> Self {
            Self {
                name,
                passes: Vec::new(),
            }
        }

        pub fn push_pass(&mut self, pass: RenderPassData<dyn Material>) {
            self.passes.push(Arc::new(RwLock::new(pass)));
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
pub struct RenderPassData<M: Material + ?Sized> {
    pub material: Arc<RwLock<M>>,
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    // Implement instancing here
}

impl RenderPassData<dyn Material> {
    pub fn set_vertices(&mut self, device: &wgpu::Device, vertices: &Vec<Vertex>) {
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.vertex_count = vertices.len() as u32;
    }

    pub fn set_indices(&mut self, device: &wgpu::Device, indices: &Vec<u32>) {
        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.index_count = indices.len() as u32;
    }
}

pub fn create_render_pass(
    state: &State,
    material: Arc<RwLock<dyn Material>>,
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

    RenderPassData {
        material: Arc::clone(&material),
        vertex_buffer,
        index_buffer,
        vertex_count,
        index_count,
    }
}
