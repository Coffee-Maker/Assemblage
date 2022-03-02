use std::cmp::min;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use glam::{IVec3, UVec3};
use noise::{NoiseFn, Perlin};
use rayon::prelude::*;

use crate::rendering::mesh::Mesh;
use crate::rendering::vertex::Vertex;
use crate::voxels::voxel_data::{voxel_shapes, VoxelData, VoxelShape};

pub const CHUNK_SIZE: u32 = 8;

pub struct VoxelScene {
    pub chunks: Arc<Mutex<HashMap<IVec3, VoxelChunk>>>,
    pub chunk_initialize_queue: Arc<Mutex<VecDeque<IVec3>>>,
    pub chunk_generation_queue: Arc<Mutex<VecDeque<IVec3>>>,
    pub chunk_submission_queue: Arc<Mutex<VecDeque<IVec3>>>,
}

impl VoxelScene {
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(Mutex::new(HashMap::default())),
            chunk_initialize_queue: Arc::new(Mutex::new(VecDeque::new())),
            chunk_generation_queue: Arc::new(Mutex::new(VecDeque::new())),
            chunk_submission_queue: Arc::new(Mutex::new(VecDeque::new())),
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
        let chunks_lock = self.chunks.lock().unwrap();
        None //chunks_lock.get(&chunk_pos)
    }

    pub fn chunk_at_mut(&mut self, position: &IVec3) -> Option<&mut VoxelChunk> {
        let chunk_pos = IVec3::new(
            position.x.div_floor(CHUNK_SIZE as i32),
            position.y.div_floor(CHUNK_SIZE as i32),
            position.z.div_floor(CHUNK_SIZE as i32),
        );
        let mut chunks_lock = self.chunks.lock().unwrap();
        None //chunks_lock.get_mut(&chunk_pos)
    }

    fn register_chunk(&self, chunk: VoxelChunk) {
        let mut chunks_lock = self.chunks.lock().unwrap();
        chunks_lock.insert(chunk.position, chunk);
    }

    pub fn initialize_chunk(&self, position: &IVec3) {
        let mut queue_lock = self.chunk_initialize_queue.lock().unwrap();
        queue_lock.push_back(*position);
    }

    pub fn process_initialization_queue(scene: Arc<VoxelScene>) {
        rayon::spawn(move || {
            loop {
                let mut queue_lock = scene.chunk_initialize_queue.lock().unwrap();
                if queue_lock.len() == 0 {
                    drop(queue_lock);
                    //println!("Initializer waiting");
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }

                println!("Initializer started");
                for _ in 0..queue_lock.len() {
                    let chunk_pos = queue_lock.pop_front().unwrap();
    
                    let mut chunk = VoxelChunk::new(chunk_pos);
                
                    // Set chunk data
                    let noise = Perlin::new();
                    let chunk_pos_scenespace = chunk.scenespace_pos();
                    chunk
                        .voxels
                        .iter_mut()
                        .enumerate()
                        .for_each(|(x, arr0)| {
                            arr0.iter_mut().enumerate().for_each(|(y, arr1)| {
                                arr1.iter_mut().enumerate().for_each(|(z, voxel)| {
                                    voxel.density = get_density(
                                        IVec3::new(
                                            chunk_pos_scenespace.x + x as i32,
                                            chunk_pos_scenespace.y + y as i32,
                                            chunk_pos_scenespace.z + z as i32,
                                        ),
                                        &noise,
                                    ) as f32;
                                    if voxel.density > 0.0 {

                                        voxel.shape = voxel_shapes::ALL;

                                    }
                                });
                            });
                        });
    
                    
                    scene.register_chunk(chunk);
    
                    let mut generation_queue_lock = scene.chunk_generation_queue.lock().unwrap();
                    generation_queue_lock.push_back(chunk_pos.clone());
                }
                
                println!("Completed initialization");
            }
        });
    }

    pub fn process_generation_queue(scene: Arc<VoxelScene>) {
        rayon::spawn(move || {
            loop {
                let mut generation_queue_lock = scene.chunk_generation_queue.lock().unwrap();
                if generation_queue_lock.len() == 0 {
                    drop(generation_queue_lock);
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                
                let mut chunks = HashMap::new();
                let mut chunks_lock = scene.chunks.lock().unwrap();
                for _ in 0..min(generation_queue_lock.len(), 50) {
                    let pos = generation_queue_lock.pop_front().unwrap();
                    chunks.insert(pos, chunks_lock.remove(&pos).unwrap());
                }
                drop(chunks_lock);
                drop(generation_queue_lock);

                for (_position, chunk) in &mut chunks {
                    chunk.generate_mesh();
                }
                
                let mut submission_queue_lock = scene.chunk_submission_queue.lock().unwrap();
                let mut chunks_lock = scene.chunks.lock().unwrap();
                for (position, chunk) in chunks {
                    chunks_lock.insert(position, chunk);
                    submission_queue_lock.push_back(position);
                }
                drop(chunks_lock);
            }
        });
    }
}

pub fn get_density(position: IVec3, noise: &Perlin) -> f64 {
    let height_offset: f32 = 20.0;
    let height_blend: f32 = 1.0;

    let mut final_density = 0.0;
    final_density += perlin_scaled(position, noise, 25.0, 200.0);
    final_density += perlin_scaled(position, noise, 10.0, 100.0);
    final_density += perlin_scaled(position, noise, 5.0, 20.0);

    final_density -= ((position.y as f32 / height_blend) - height_offset) as f64;
    final_density
}

pub fn perlin_scaled(position: IVec3, noise: &Perlin, amplitude: f32, wavelength: f32) -> f64 {
    let scaled_position = position.as_vec3() / wavelength;
    noise.get([
        scaled_position.x as f64,
        scaled_position.y as f64,
        scaled_position.z as f64,
    ]) * amplitude as f64
}

#[derive(Clone)]
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
                density: 0.0,
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
    let global_position = position + chunk.scenespace_pos();

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
        vertices.reserve(4);
        
        let color = [0.8, 0.5, 0.3];

        // v0
        vertices.push(Vertex {
            position: [
                quad_verts[0][0] + f_position.x,
                quad_verts[0][1] + f_position.y,
                quad_verts[0][2] + f_position.z,
            ],
            color: color,
            normal,
            uv: [0.0, 0.0],
        });

        // v1
        vertices.push(Vertex {
            position: [
                quad_verts[1][0] + f_position.x,
                quad_verts[1][1] + f_position.y,
                quad_verts[1][2] + f_position.z,
            ],
            color: color,
            normal,
            uv: [1.0, 0.0],
        });

        // v2
        vertices.push(Vertex {
            position: [
                quad_verts[2][0] + f_position.x,
                quad_verts[2][1] + f_position.y,
                quad_verts[2][2] + f_position.z,
            ],
            color: color,
            normal,
            uv: [0.0, 1.0],
        });

        // v3
        vertices.push(Vertex {
            position: [
                quad_verts[3][0] + f_position.x,
                quad_verts[3][1] + f_position.y,
                quad_verts[3][2] + f_position.z,
            ],
            color: color,
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
