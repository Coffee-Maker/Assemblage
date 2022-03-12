use crate::{next_id, rendering::vertex::Vertex};
use bus::Bus;
use core::fmt::Debug;
use glam::Vec3;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use super::asset::{Asset, AssetChangeType};

pub struct Mesh {
    pub vertex_count: usize,
    pub index_count: usize,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    change_channel: Bus<AssetChangeType>,
    id: u64,
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            vertex_count: 0,
            index_count: 0,
            vertices: Vec::new(),
            indices: Vec::new(),
            change_channel: Bus::new(100), // Magic number, I don't know what length this should be
            id: next_id(),
        }
    }

    pub fn append_custom(
        mut self,
        vertices: Vec<[f32; 3]>,
        indices: Vec<u32>,
        normal: [f32; 3],
    ) -> Mesh {
        let index_offset = self.vertices.len() as u32;
        self.indices.reserve(indices.len());
        indices.iter().for_each(|i| {
            self.indices.push(index_offset + i);
        });

        self.vertices.reserve(vertices.len());

        let color = [0.5, 0.3, 0.2];

        vertices.iter().for_each(|position| {
            self.vertices.push(Vertex {
                position: *position,
                color,
                normal,
                uv: [0.0, 0.0],
            }) // TODO: Add UVs
        });

        self.send_changes(AssetChangeType::Modified);
        self
    }

    pub fn append_quad(mut self, quad_verts: [[f32; 3]; 4], normal: [f32; 3]) -> Mesh {
        let index_offset = self.vertices.len() as u32;
        self.indices.append(&mut vec![
            index_offset,
            index_offset + 2,
            index_offset + 1,
            index_offset + 2,
            index_offset + 3,
            index_offset + 1,
        ]);
        self.vertices.reserve(4);

        let color = [0.5, 0.3, 0.2];

        // v0
        self.vertices.push(Vertex {
            position: [quad_verts[0][0], quad_verts[0][1], quad_verts[0][2]],
            color: color,
            normal,
            uv: [0.0, 0.0],
        });

        // v1
        self.vertices.push(Vertex {
            position: [quad_verts[1][0], quad_verts[1][1], quad_verts[1][2]],
            color: color,
            normal,
            uv: [1.0, 0.0],
        });

        // v2
        self.vertices.push(Vertex {
            position: [quad_verts[2][0], quad_verts[2][1], quad_verts[2][2]],
            color: color,
            normal,
            uv: [0.0, 1.0],
        });

        // v3
        self.vertices.push(Vertex {
            position: [quad_verts[3][0], quad_verts[3][1], quad_verts[3][2]],
            color: color,
            normal,
            uv: [1.0, 1.0],
        });

        self.send_changes(AssetChangeType::Modified);
        self
    }

    pub fn append_tri(mut self, quad_verts: [[f32; 3]; 3], normal: [f32; 3]) -> Mesh {
        let index_offset = self.vertices.len() as u32;
        self.indices
            .append(&mut vec![index_offset, index_offset + 2, index_offset + 1]);
        self.vertices.reserve(4);

        let color = [0.8, 0.5, 0.3];

        // v0
        self.vertices.push(Vertex {
            position: [quad_verts[0][0], quad_verts[0][1], quad_verts[0][2]],
            color: color,
            normal,
            uv: [0.0, 0.0],
        });

        // v1
        self.vertices.push(Vertex {
            position: [quad_verts[1][0], quad_verts[1][1], quad_verts[1][2]],
            color: color,
            normal,
            uv: [1.0, 0.0],
        });

        // v2
        self.vertices.push(Vertex {
            position: [quad_verts[2][0], quad_verts[2][1], quad_verts[2][2]],
            color: color,
            normal,
            uv: [0.0, 1.0],
        });

        self.vertex_count = self.vertices.len();
        self.index_count = self.indices.len();
        self.send_changes(AssetChangeType::Modified);
        self
    }

    pub fn append_vertices(&mut self, vertices: &mut Vec<Vertex>) {
        self.vertices.append(vertices);
        self.vertex_count = self.vertices.len();
        self.send_changes(AssetChangeType::Modified);
    }

    pub fn append_indices(&mut self, indices: &mut Vec<u32>) {
        self.indices.append(indices);
        self.index_count = self.indices.len();
        self.send_changes(AssetChangeType::Modified);
    }

    pub fn append_indices_with_offset(&mut self, indices: &mut Vec<u32>, offset: u32) {
        indices.par_iter_mut().for_each(|i| *i += offset);
        self.indices.append(indices);
        self.index_count = self.indices.len();
        self.send_changes(AssetChangeType::Modified);
    }

    pub fn set_vertices(&mut self, vertices: Vec<Vertex>) {
        self.vertex_count = vertices.len();
        self.vertices = vertices;
        self.send_changes(AssetChangeType::Modified);
    }

    pub fn set_indices(&mut self, indices: Vec<u32>) {
        self.index_count = indices.len();
        self.indices = indices;
        self.send_changes(AssetChangeType::Modified);
    }

    pub fn get_vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn get_indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn offset_vertices(&mut self, offset: &Vec3) {
        self.vertices.iter_mut().for_each(|vertex| {
            vertex.position[0] += offset.x;
            vertex.position[1] += offset.y;
            vertex.position[2] += offset.z;
        });
    }
}

impl Asset for Mesh {
    fn get_change_receiver(&mut self) -> bus::BusReader<super::asset::AssetChangeType> {
        self.change_channel.add_rx()
    }

    fn send_changes(&mut self, change_type: AssetChangeType) {
        self.change_channel.broadcast(change_type);
    }

    fn get_id(&self) -> u64 {
        self.id
    }
}

impl Clone for Mesh {
    fn clone(&self) -> Self {
        Self {
            vertex_count: self.vertex_count.clone(),
            index_count: self.index_count.clone(),
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),
            change_channel: Bus::new(100),
            id: self.id,
        }
    }
}

impl Debug for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mesh")
            .field("vertex_count", &self.vertex_count)
            .field("index_count", &self.index_count)
            .field("vertices", &self.vertices)
            .field("indices", &self.indices)
            .finish()
    }
}
