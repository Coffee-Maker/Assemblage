use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use dashmap::DashMap;
use flume::{Receiver, Sender};
use glam::{IVec3, UVec3};
use noise::{NoiseFn, Perlin};

use crate::rendering::mesh::Mesh;
use crate::rendering::vertex::Vertex;
use crate::voxels::voxel_data::VoxelData;
use crate::voxels::voxel_shapes::voxel_shapes;

use super::voxel_mesh::get_voxel_mesh;
use super::voxel_shapes::{voxel_directions, voxel_orientations, VoxelDirection, VoxelShape};

pub const CHUNK_SIZE: u32 = 16;
type ChunkData = Arc<DashMap<IVec3, VoxelChunk, ahash::RandomState>>;

pub struct VoxelScene {
    pub chunks: ChunkData,

    initialization_channel: (
        Sender<(IVec3, Option<Sender<IVec3>>)>,
        Receiver<(IVec3, Option<Sender<IVec3>>)>,
    ),
    generation_channel: (Sender<IVec3>, Receiver<IVec3>),
    generation_pre_processor_channel: (Sender<IVec3>, Receiver<IVec3>),
}

impl VoxelScene {
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(DashMap::default()),
            initialization_channel: flume::unbounded(),
            generation_channel: flume::unbounded(),
            generation_pre_processor_channel: flume::unbounded(),
        }
    }

    pub fn voxel_at(&self, position: &IVec3) -> Option<VoxelData> {
        let chunk_pos = Self::chunk_at(position);
        self.chunks
            .get(&chunk_pos)
            .map(|chunk| chunk.voxel_scenespace_at(position).unwrap().to_owned())
    }

    pub fn chunk_at(position: &IVec3) -> IVec3 {
        IVec3::new(
            position.x.div_floor(CHUNK_SIZE as i32),
            position.y.div_floor(CHUNK_SIZE as i32),
            position.z.div_floor(CHUNK_SIZE as i32),
        )
    }

    pub fn setup_chunk_processors(&mut self, mesh_sender: Sender<(IVec3, Mesh)>) {
        for _i in 0..2 {
            VoxelScene::initialization_processor(
                Arc::clone(&self.chunks),
                self.initialization_channel.1.clone(),
            );
        }

        for _i in 0..2 {
            let chunks_clone = Arc::clone(&self.chunks);
            VoxelScene::generation_processor(
                chunks_clone,
                self.generation_channel.1.clone(),
                mesh_sender.clone(),
            );
        }

        for _i in 0..2 {
            let chunks_clone = Arc::clone(&self.chunks);
            VoxelScene::generation_pre_processor(
                chunks_clone,
                self.generation_pre_processor_channel.1.clone(),
                self.initialization_channel.0.clone(),
                self.generation_channel.0.clone(),
            );
        }
    }

    pub fn initialize_and_generate_chunk(&self, position: IVec3) {
        self.initialization_channel
            .0
            .send((
                position,
                Some(self.generation_pre_processor_channel.0.clone()),
            ))
            .unwrap();
        self.generation_pre_processor_channel
            .0
            .send(position)
            .unwrap();
    }

    pub fn initialization_processor(
        chunks: ChunkData,
        pos_receiver: Receiver<(IVec3, Option<Sender<IVec3>>)>,
    ) {
        rayon::spawn(move || {
            println!("Started initialization processor");
            let mut initialized_chunks = Vec::new();
            loop {
                let request = pos_receiver.recv().unwrap();
                if initialized_chunks.contains(&request.0) {
                    continue;
                }
                let mut chunk = VoxelChunk::new(request.0);

                // Set chunk data
                let noise = Perlin::new();
                let chunk_pos_scenespace = chunk.scenespace_pos();
                chunk
                    .voxels
                    .iter_mut()
                    .enumerate()
                    .for_each(|(index, voxel)| {
                        let voxel_pos = index_to_pos(index as u32);
                        let density = get_density(
                            IVec3::new(
                                chunk_pos_scenespace.x + voxel_pos.x as i32,
                                chunk_pos_scenespace.y + voxel_pos.y as i32,
                                chunk_pos_scenespace.z + voxel_pos.z as i32,
                            ),
                            &noise,
                        ) as f32;
                        if density > 0.0 {
                            chunk.is_empty = false;
                            voxel.shape = if density < 0.5 {
                                voxel_shapes::SLAB.oriented(voxel_orientations::NORTH)
                            } else {
                                voxel_shapes::CUBE
                            };
                            voxel.id = 1;
                        }
                    });

                initialized_chunks.push(request.0);
                chunks.insert(request.0, chunk);
                request.1.map(|s| s.send(request.0));
            }
        });
    }

    pub fn generation_processor(
        chunks: ChunkData,
        pos_receiver: Receiver<IVec3>,
        mesh_sender: Sender<(IVec3, Mesh)>,
    ) {
        rayon::spawn(move || {
            println!("Started generation processor");
            loop {
                let chunk_pos = pos_receiver.recv().unwrap();

                let chunk = (*chunks.get(&chunk_pos).unwrap()).clone();
                let chunks_clone = Arc::clone(&chunks);
                let mesh = chunk.generate_mesh(chunks_clone);
                mesh_sender.send((chunk_pos, mesh)).unwrap();
            }
        });
    }

    pub fn generation_pre_processor(
        chunks: ChunkData,
        pos_receiver: Receiver<IVec3>,
        initialization_sender: Sender<(IVec3, Option<Sender<IVec3>>)>,
        pos_sender: Sender<IVec3>,
    ) {
        rayon::spawn(move || {
            println!("Started generation preprocessor");
            // store a list of chunk positions
            let mut chunks_to_generate = VecDeque::new();
            loop {
                let chunk_pos = pos_receiver.try_recv();
                // if we did not receive a chunk, check the chunks_to_generate list
                let chunk_pos = match chunk_pos {
                    Ok(chunk_pos) => chunk_pos,
                    Err(_e) => {
                        if chunks_to_generate.len() > 0 {
                            chunks_to_generate.pop_front().unwrap()
                        } else {
                            thread::sleep(Duration::from_millis(10));
                            continue;
                        }
                    }
                };

                // get a list of neighbours
                let mut failed = false;
                for direction in voxel_directions::ALL {
                    let neighbour_pos = chunk_pos + direction.as_vec();
                    if !chunks.contains_key(&neighbour_pos) {
                        failed = true;
                        initialization_sender.send((neighbour_pos, None)).unwrap();
                    }
                }

                // if all neighbours are generated, schedule the chunk to be generated
                if !failed {
                    pos_sender.send(chunk_pos).unwrap();
                } else {
                    chunks_to_generate.push_back(chunk_pos);
                }
            }
        });
    }
}

pub fn get_density(position: IVec3, noise: &Perlin) -> f64 {
    let height_offset: f32 = 10.0;
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
    voxels: Vec<VoxelData>,
}

impl VoxelChunk {
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            is_empty: true,
            voxels: vec![
                VoxelData {
                    shape: voxel_shapes::CUBE,
                    state: 0,
                    id: 0,
                };
                (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize
            ],
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
        &self.voxels.get(pos_to_index(&position) as usize).unwrap()
    }

    pub fn voxel_at_mut(&mut self, position: &UVec3) -> &mut VoxelData {
        self.voxels
            .get_mut(pos_to_index(&position) as usize)
            .unwrap()
    }

    pub fn set_voxel_shape(&mut self, position: &UVec3, shape: VoxelShape) {
        self.voxel_at_mut(position).shape = shape
    }

    pub fn generate_mesh(&self, scene_chunks: ChunkData) -> Mesh {
        let mut vertices = vec![];
        let mut indices = vec![];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let pos = UVec3::new(x, y, z);
                    let voxel = self.voxel_at(&pos);
                    if voxel.id != 0 {
                        // Voxel is not air
                        let scene_chunks_clone = Arc::clone(&scene_chunks);
                        generate_faces(
                            voxel,
                            scene_chunks_clone,
                            self,
                            &pos,
                            &mut vertices,
                            &mut indices,
                        );
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

fn index_to_pos(index: u32) -> UVec3 {
    let x = index / (CHUNK_SIZE * CHUNK_SIZE);
    let y = index % (CHUNK_SIZE * CHUNK_SIZE) / CHUNK_SIZE;
    let z = index % CHUNK_SIZE;
    UVec3::new(x, y, z)
}

pub fn pos_to_index(pos: &UVec3) -> u32 {
    (pos.x * CHUNK_SIZE * CHUNK_SIZE) + (pos.y * CHUNK_SIZE) + pos.z
}

#[inline(always)]
fn generate_faces(
    voxel: &VoxelData,
    scene_chunks: ChunkData,
    chunk: &VoxelChunk,
    position: &UVec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
) {
    let position = position.as_ivec3();
    let f_position = position.as_vec3();
    let global_position = position + chunk.scenespace_pos();

    let face_check = |direction: VoxelDirection| -> bool {
        let sample_position = global_position + direction.as_vec();
        let neighbour = chunk.voxel_scenespace_at(&sample_position).map_or_else(
            || {
                scene_chunks
                    .get(&VoxelScene::chunk_at(&sample_position))
                    .map_or(None, |chunk| {
                        chunk.voxel_scenespace_at(&sample_position).cloned()
                    })
            },
            |&voxel| Some(voxel),
        );
        neighbour.map_or(true, |neighbour| {
            neighbour.id == 0
                || !neighbour
                    .shape
                    .face_contains(direction.flip(), (voxel.shape, direction))
        })
    };

    let mut append_mesh = |mesh: &Mesh| {
        let index_offset = vertices.len();

        let flip_x = voxel.shape.extract_flip_x();
        let flip_y = voxel.shape.extract_flip_y();
        let flip_z = voxel.shape.extract_flip_z();
        let flip_count = (flip_x as u32 + flip_y as u32 + flip_z as u32) % 2;

        indices.reserve(mesh.indices.len());
        for index in 0..mesh.indices.len() {
            indices.push(
                (if (flip_count & 1) == 0 {
                    mesh.indices[index]
                } else {
                    mesh.indices[mesh.indices.len() - index - 1]
                }) + index_offset as u32,
            );
        }

        vertices.reserve(mesh.vertices.len());

        mesh.vertices.iter().for_each(|v| {
            let mut vert = v.clone();
            if flip_x {
                vert.position[0] *= -1.0;
                vert.normal[0] *= -1.0;
            }
            if voxel.shape.extract_flip_y() {
                vert.position[1] *= -1.0;
                vert.normal[1] *= -1.0;
            }
            if voxel.shape.extract_flip_z() {
                vert.position[2] *= -1.0;
                vert.normal[2] *= -1.0;
            }
            if voxel.shape.extract_rotate_x() {
                (vert.position[1], vert.position[2]) = (vert.position[2], -vert.position[1]);
                (vert.normal[1], vert.normal[2]) = (vert.normal[2], -vert.normal[1]);
            }
            if voxel.shape.extract_rotate_z() {
                (vert.position[0], vert.position[1]) = (vert.position[1], -vert.position[0]);
                (vert.normal[0], vert.normal[1]) = (vert.normal[1], -vert.normal[0]);
            }
            vert.position[0] += f_position.x;
            vert.position[1] += f_position.y;
            vert.position[2] += f_position.z;
            vertices.push(vert);
        });
    };

    let shape_mesh = get_voxel_mesh(voxel.shape);

    append_mesh(&shape_mesh.always);

    let orientations = VoxelDirection::get_oriented_directions(voxel.shape.extract_orientation());
    // North
    if face_check(orientations.get_direction(voxel_directions::NORTH)) {
        append_mesh(&shape_mesh.north);
    }

    // South
    if face_check(orientations.get_direction(voxel_directions::SOUTH)) {
        append_mesh(&shape_mesh.south);
    }

    // East
    if face_check(orientations.get_direction(voxel_directions::EAST)) {
        append_mesh(&shape_mesh.east);
    }

    // West
    if face_check(orientations.get_direction(voxel_directions::WEST)) {
        append_mesh(&shape_mesh.west);
    }

    // Up
    if face_check(orientations.get_direction(voxel_directions::UP)) {
        append_mesh(&shape_mesh.top);
    }

    // Down
    if face_check(orientations.get_direction(voxel_directions::DOWN)) {
        append_mesh(&shape_mesh.bottom);
    }
}
