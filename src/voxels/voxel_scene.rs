use std::collections::{HashMap, VecDeque};

use glam::{IVec3, UVec3};
use noise::{NoiseFn, Perlin};
use rayon::prelude::*;

use crate::rendering::mesh::Mesh;
use crate::rendering::vertex::Vertex;
use crate::voxels::voxel_data::{voxel_shapes, VoxelData, VoxelShape};

pub const CHUNK_SIZE: u32 = 8;

pub struct VoxelScene {
    pub chunks: HashMap<IVec3, VoxelChunk>,
    chunk_initialize_queue: VecDeque<IVec3>,
}

impl VoxelScene {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::default(),
            chunk_initialize_queue: VecDeque::new(),
        }
    }

    pub fn voxel_at(&self, position: &IVec3) -> Option<&VoxelData> {
        self.chunk_at(position)
            .map(|chunk| chunk.voxel_scenespace_at(position).unwrap())
    }

    pub fn voxel_at_mut(&mut self, position: &IVec3) -> Option<&mut VoxelData> {
        self.chunk_at_mut(position)
            .map(|chunk| chunk.voxel_scenespace_at_mut(position).unwrap())
    }

    pub fn chunk_at(&self, position: &IVec3) -> Option<&VoxelChunk> {
        let chunk_pos = IVec3::new(
            position.x.div_floor(CHUNK_SIZE as i32),
            position.y.div_floor(CHUNK_SIZE as i32),
            position.z.div_floor(CHUNK_SIZE as i32),
        );
        self.chunks.get(&chunk_pos)
    }

    pub fn chunk_at_mut(&mut self, position: &IVec3) -> Option<&mut VoxelChunk> {
        let chunk_pos = IVec3::new(
            position.x.div_floor(CHUNK_SIZE as i32),
            position.y.div_floor(CHUNK_SIZE as i32),
            position.z.div_floor(CHUNK_SIZE as i32),
        );
        self.chunks.get_mut(&chunk_pos)
    }

    fn register_chunk(&mut self, chunk: VoxelChunk) {
        self.chunks.insert(chunk.position, chunk);
    }

    pub fn initialize_chunk(&mut self, position: &IVec3) {
        self.chunk_initialize_queue.push_back(*position);
    }

    pub async fn process_initialization_queue(&mut self) {
        // Build and register chunk
        while self.chunk_initialize_queue.len() > 0 {
            let chunk_pos = self.chunk_initialize_queue.pop_front().unwrap();
            let chunk = VoxelChunk::new(chunk_pos);
            self.register_chunk(chunk);
        }

        // Set chunk data
        let noise = Perlin::new();
        self.chunks.par_iter_mut().for_each(|(_chunk_pos, chunk)| {
            let chunk_pos_scenespace = chunk.scenespace_pos();
            chunk
                .voxels
                .iter_mut()
                .enumerate()
                .for_each(|(x, arr0)| {
                    arr0.iter_mut().enumerate().for_each(|(y, arr1)| {
                        arr1.iter_mut().enumerate().for_each(|(z, voxel)| {
                            voxel.shape = if get_density(
                                IVec3::new(
                                    chunk_pos_scenespace.x + x as i32,
                                    chunk_pos_scenespace.y + y as i32,
                                    chunk_pos_scenespace.z + z as i32,
                                ),
                                &noise,
                            ) < 0.5
                            {
                                voxel_shapes::ALL
                            } else {
                                voxel_shapes::EMPTY
                            }
                        });
                    });
                });

            chunk.generate_mesh();
        });
    }
}

pub fn get_density(position: IVec3, noise: &Perlin) -> f64 {
    let scaled_position = position.as_vec3() * 0.1;
    noise.get([
        scaled_position.x as f64,
        scaled_position.y as f64,
        scaled_position.z as f64,
    ]) + (scaled_position.y) as f64
}

pub struct VoxelChunk {
    pub position: IVec3,
    pub mesh: Mesh,
    voxels: [[[VoxelData; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
}

impl VoxelChunk {
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            mesh: Mesh::new(),
            voxels: [[[VoxelData {
                shape: voxel_shapes::EMPTY,
            }; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
        }
    }

    pub fn voxel_scenespace_at_mut(&mut self, position: &IVec3) -> Option<&mut VoxelData> {
        let localized_pos = *position - (self.position * CHUNK_SIZE as i32);
        if localized_pos.x >= CHUNK_SIZE as i32
            || localized_pos.y >= CHUNK_SIZE as i32
            || localized_pos.z >= CHUNK_SIZE as i32
            || localized_pos.x < 0
            || localized_pos.y < 0
            || localized_pos.z < 0
        {
            return None;
        }
        Some(self.voxel_at_mut(&localized_pos.as_uvec3()))
    }

    pub fn voxel_scenespace_at(&self, position: &IVec3) -> Option<&VoxelData> {
        let localized_pos = *position - (self.position * CHUNK_SIZE as i32);
        if localized_pos.x >= CHUNK_SIZE as i32
            || localized_pos.y >= CHUNK_SIZE as i32
            || localized_pos.z >= CHUNK_SIZE as i32
            || localized_pos.x < 0
            || localized_pos.y < 0
            || localized_pos.z < 0
        {
            return None;
        }
        Some(self.voxel_at(&localized_pos.as_uvec3()))
    }

    pub fn voxel_at(&self, position: &UVec3) -> &VoxelData {
        &self.voxels[position.x as usize][position.y as usize][position.z as usize]
    }

    pub fn voxel_at_mut(&mut self, position: &UVec3) -> &mut VoxelData {
        &mut self.voxels[position.x as usize][position.y as usize][position.z as usize]
    }

    pub fn set_voxel_shape(&mut self, position: &UVec3, shape: VoxelShape) {
        self.voxel_at_mut(position).shape = shape
    }

    pub fn generate_mesh(&mut self) {
        let mut vertices = vec![];
        let mut indices = vec![];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let pos = UVec3::new(x, y, z);
                    if self.voxel_at(&pos).shape != voxel_shapes::EMPTY {
                        generate_faces(self, &pos, &mut vertices, &mut indices);
                    }
                }
            }
        }

        let mut mesh = Mesh::new();

        mesh.vertices.append(&mut vertices);
        mesh.indices.append(&mut indices);

        self.mesh = mesh;
    }

    pub fn scenespace_pos(&self) -> IVec3 {
        self.position * CHUNK_SIZE as i32
    }
}

#[inline(always)]
fn generate_faces(
    chunk: &VoxelChunk,
    position: &UVec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let position = position.as_ivec3();
    let f_position = position.as_vec3();
    let global_position = position + (chunk.position * CHUNK_SIZE as i32);

    let face_check = |offset: IVec3, space_requirement: VoxelShape| {
        chunk
            .voxel_scenespace_at(&(global_position + offset))
            .map_or(true, |voxel| !voxel.shape.contains(space_requirement))
    };

    let mut build_quad = |quad_verts: &mut [[f32; 3]; 4], normal: [f32; 3]| {
        let offset = vertices.len() as u32;
        indices.append(&mut vec![
            offset,
            offset + 2,
            offset + 1,
            offset + 1,
            offset + 2,
            offset + 3,
        ]);

        // v0
        vertices.push(Vertex {
            position: [
                quad_verts[0][0] + f_position.x as f32,
                quad_verts[0][1] + f_position.y as f32,
                quad_verts[0][2] + f_position.z as f32,
            ],
            color: [1.0, 1.0, 1.0],
            normal,
            uv: [0.0, 0.0],
        });

        // v1
        vertices.push(Vertex {
            position: [
                quad_verts[1][0] + f_position.x as f32,
                quad_verts[1][1] + f_position.y as f32,
                quad_verts[1][2] + f_position.z as f32,
            ],
            color: [1.0, 1.0, 1.0],
            normal,
            uv: [1.0, 0.0],
        });

        // v2
        vertices.push(Vertex {
            position: [
                quad_verts[2][0] + f_position.x as f32,
                quad_verts[2][1] + f_position.y as f32,
                quad_verts[2][2] + f_position.z as f32,
            ],
            color: [1.0, 1.0, 1.0],
            normal,
            uv: [0.0, 1.0],
        });

        // v3
        vertices.push(Vertex {
            position: [
                quad_verts[3][0] + f_position.x as f32,
                quad_verts[3][1] + f_position.y as f32,
                quad_verts[3][2] + f_position.z as f32,
            ],
            color: [1.0, 1.0, 1.0],
            normal,
            uv: [1.0, 1.0],
        });
    };

    // North
    if face_check(IVec3::Z, voxel_shapes::SOUTH) {
        build_quad(
            &mut [
                [1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            [0.0, 0.0, 1.0],
        );
    }

    // South
    if face_check(-IVec3::Z, voxel_shapes::NORTH) {
        build_quad(
            &mut [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            [0.0, 0.0, -1.0],
        );
    }

    // East
    if face_check(IVec3::X, voxel_shapes::WEST) {
        build_quad(
            &mut [
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
            ],
            [1.0, 0.0, 0.0],
        );
    }

    // West
    if face_check(-IVec3::X, voxel_shapes::EAST) {
        build_quad(
            &mut [
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0],
            ],
            [-1.0, 0.0, 0.0],
        );
    }

    // Top
    if face_check(IVec3::Y, voxel_shapes::BOTTOM) {
        build_quad(
            &mut [
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
            ],
            [0.0, 1.0, 0.0],
        );
    }

    // Bottom
    if face_check(-IVec3::Y, voxel_shapes::TOP) {
        build_quad(
            &mut [
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
            ],
            [0.0, -1.0, 0.0],
        );
    }
}
