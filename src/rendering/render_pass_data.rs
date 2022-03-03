use super::{vertex::Vertex};
use wgpu::util::DeviceExt;

pub struct RenderPassData {
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub diffuse_bind_group: wgpu::BindGroup,
}

impl RenderPassData {
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
