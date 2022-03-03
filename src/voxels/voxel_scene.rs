use std::collections::{HashMap};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use flume::{Receiver, Sender};
use glam::{IVec3, UVec3};
use noise::{NoiseFn, Perlin};

use crate::rendering::mesh::Mesh;
use crate::rendering::vertex::Vertex;
use crate::voxels::voxel_data::{voxel_shapes, VoxelData, VoxelShape};

pub const CHUNK_SIZE: u32 = 16;

pub struct VoxelScene {
    pub chunks: Arc<Mutex<HashMap<IVec3, VoxelChunk>>>,

    initialization_channel: (Sender<IVec3>, Receiver<IVec3>),
    registration_channel: (Sender<VoxelChunk>, Receiver<VoxelChunk>),
    generation_channel: (Sender<IVec3>, Receiver<IVec3>),
}

impl VoxelScene {
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(Mutex::new(HashMap::default())),
            initialization_channel: flume::unbounded(),
            registration_channel: flume::unbounded(),
            generation_channel: flume::unbounded(),

        }
    }

    pub fn voxel_at(&self, position: &IVec3) -> Option<VoxelData> {
        let chunk_pos = Self::chunk_at(position);
        let chunk_lock = self.chunks.lock().unwrap();
        chunk_lock.get(&chunk_pos).map(|chunk| chunk.voxel_scenespace_at(position).unwrap().to_owned())
    }

    pub fn chunk_at(position: &IVec3) -> IVec3 {
        IVec3::new(
            position.x.div_floor(CHUNK_SIZE as i32),
            position.y.div_floor(CHUNK_SIZE as i32),
            position.z.div_floor(CHUNK_SIZE as i32),
        )
    }

    pub fn setup_chunk_processors(&mut self, mesh_sender: Sender<(IVec3, Mesh)>) {
        for _i in 0..5 {
            VoxelScene::initialization_processor(self.initialization_channel.1.clone(), self.registration_channel.0.clone());
        }

        for _i in 0..5 {
            let chunks_clone = Arc::clone(&self.chunks);
            VoxelScene::generation_processor(chunks_clone, self.generation_channel.1.clone(), mesh_sender.clone());
        }

        for _ in 0..1 {
            let chunks_clone = Arc::clone(&self.chunks);
            VoxelScene::registration_processor(self.registration_channel.1.clone(), chunks_clone, self.generation_channel.0.clone());
        }
    }

    pub fn initialize_chunk(&self, position: IVec3) {
        self.initialization_channel.0.send(position).unwrap();
    }

    pub fn initialization_processor(pos_receiver: Receiver<IVec3>, chunk_sender: Sender<VoxelChunk>) {
        rayon::spawn(move || {
            println!("Started initialization processor");
            loop {
                let chunk_pos = pos_receiver.recv().unwrap();

                let mut chunk = VoxelChunk::new(chunk_pos);

                // Set chunk data
                let noise = Perlin::new();
                let chunk_pos_scenespace = chunk.scenespace_pos();
                chunk.voxels.iter_mut().enumerate().for_each(|(x, arr0)| {
                    arr0.iter_mut().enumerate().for_each(|(y, arr1)| {
                        arr1.iter_mut().enumerate().for_each(|(z, voxel)| {
                            let density = get_density(
                                IVec3::new(
                                    chunk_pos_scenespace.x + x as i32,
                                    chunk_pos_scenespace.y + y as i32,
                                    chunk_pos_scenespace.z + z as i32,
                                ),
                                &noise,
                            ) as f32;
                            if density > 0.0 {
                                chunk.is_empty = false;
                                voxel.occlussion_shape = voxel_shapes::ALL;
                            }
                        });
                    });
                });

                chunk_sender.send(chunk).unwrap();
            }
        });
    }

    pub fn registration_processor(chunk_receiver: Receiver<VoxelChunk>, chunks: Arc<Mutex<HashMap<IVec3, VoxelChunk>>>, pos_sender: Sender<IVec3>) {
        rayon::spawn(move|| {
            loop {
                let chunk = chunk_receiver.recv().unwrap();
                let pos = chunk.position;
                let mut chunks_lock = chunks.lock().unwrap();
                chunks_lock.insert(pos, chunk);
                pos_sender.send(pos).unwrap();
            }
        });
    }

    pub fn generation_processor(chunks: Arc<Mutex<HashMap<IVec3, VoxelChunk>>>, pos_receiver: Receiver<IVec3>, mesh_sender: Sender<(IVec3, Mesh)>) {
        rayon::spawn(move || {
            thread::sleep(Duration::from_secs(2)); // Dumb way to wait for neighbours to have data
            println!("Started generation processor");
            loop {
                let chunk_pos = pos_receiver.recv().unwrap();
                let chunks_lock = chunks.lock().unwrap();
                let chunk = (*chunks_lock.get(&chunk_pos).unwrap()).clone();
                drop(chunks_lock);
                let chunks_clone = Arc::clone(&chunks);
                let mesh = chunk.generate_mesh(chunks_clone);
                mesh_sender.send((chunk_pos, mesh)).unwrap();
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
    pub is_empty: bool,
    voxels: [[[VoxelData; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
}

impl VoxelChunk {
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            is_empty: true,
            voxels: [[[VoxelData {
                occlussion_shape: voxel_shapes::EMPTY,
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
        self.voxel_at_mut(position).occlussion_shape = shape
    }

    pub fn generate_mesh(&self, scene_chunks: Arc<Mutex<HashMap<IVec3, VoxelChunk>>>) -> Mesh {
        let mut vertices = vec![];
        let mut indices = vec![];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let pos = UVec3::new(x, y, z);
                    if self.voxel_at(&pos).occlussion_shape != voxel_shapes::EMPTY {
                        let scene_chunks_clone = Arc::clone(&scene_chunks);
                        generate_faces(scene_chunks_clone, self, &pos, &mut vertices, &mut indices);
                    }
                }
            }
        }

        let mut mesh = Mesh::new();

        mesh.vertices.append(&mut vertices);
        mesh.indices.append(&mut indices);

        mesh
    }

    pub fn scenespace_pos(&self) -> IVec3 {
        self.position * CHUNK_SIZE as i32
    }
}

#[inline(always)]
fn generate_faces(
    scene_chunks: Arc<Mutex<HashMap<IVec3, VoxelChunk>>>,
    chunk: &VoxelChunk,
    position: &UVec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let position = position.as_ivec3();
    let f_position = position.as_vec3();
    let global_position = position + chunk.scenespace_pos();

    let face_check = |offset: IVec3, space_requirement: VoxelShape| {
        let sample_position = global_position + offset;
        chunk
            .voxel_scenespace_at(&sample_position)
            .map_or_else(|| {
                let scene_chunks_lock = scene_chunks.lock().unwrap();
                scene_chunks_lock.get(&VoxelScene::chunk_at(&sample_position)).map_or(false, |chunk| {
                    !chunk.voxel_scenespace_at(&sample_position).unwrap().occlussion_shape.contains(space_requirement)
                })
            }, |voxel| !voxel.occlussion_shape.contains(space_requirement))
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
