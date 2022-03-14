use std::collections::VecDeque;
use std::sync::Arc;

use dashmap::{DashMap, DashSet};
use flume::{Receiver, Sender};
use glam::{IVec3, UVec3};
use rayon::ThreadPool;
use simdnoise::NoiseBuilder;

use crate::asset_types::mesh::Mesh;
use crate::rendering::vertex::Vertex;
use crate::voxels::voxel_data::VoxelData;
use crate::voxels::voxel_shapes::voxel_shape;

use super::voxel_mesh::get_voxel_mesh;
use super::voxel_registry;
use super::voxel_shapes::{voxel_directions, VoxelDirection, VoxelShape};

pub const CHUNK_SIZE: u32 = 16;
type ChunkMap = Arc<DashMap<IVec3, VoxelChunk, ahash::RandomState>>;

pub struct VoxelScene {
    pub chunks: ChunkMap,
    initialization_queue: Arc<DashSet<IVec3>>,
    initialization_channel: (
        Sender<(IVec3, Option<Sender<IVec3>>)>,
        Receiver<(IVec3, Option<Sender<IVec3>>)>,
    ),
    generation_channel: (Sender<IVec3>, Receiver<IVec3>),
    generation_pre_processor_channel: (Sender<IVec3>, Receiver<IVec3>),
    thread_pool: ThreadPool,
}

impl VoxelScene {
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(DashMap::default()),
            initialization_queue: Arc::new(DashSet::default()),
            initialization_channel: flume::unbounded(),
            generation_channel: flume::unbounded(),
            generation_pre_processor_channel: flume::unbounded(),
            thread_pool: rayon::ThreadPoolBuilder::new()
                .num_threads(8)
                .build()
                .unwrap(),
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

    pub fn request_initialize_chunk(
        queue: Arc<DashSet<IVec3>>,
        sender: Sender<(IVec3, Option<Sender<IVec3>>)>,
        request: (IVec3, Option<Sender<IVec3>>),
    ) {
        if queue.contains(&request.0) {
            return;
        }
        queue.insert(request.0);
        sender.send(request).unwrap();
    }

    pub fn setup_chunk_processors(&mut self, mesh_sender: Sender<(IVec3, Mesh)>) {
        for _i in 0..3 {
            let chunks_clone = Arc::clone(&self.chunks);
            let initialization_channel_receiver = self.initialization_channel.1.clone();
            self.thread_pool.spawn(move || {
                VoxelScene::initialization_processor(chunks_clone, initialization_channel_receiver);
            });
        }

        for _i in 0..3 {
            let chunks_clone = Arc::clone(&self.chunks);
            let generation_channel_receiver = self.generation_channel.1.clone();
            let mesh_sender_clone = mesh_sender.clone();
            self.thread_pool.spawn(move || {
                VoxelScene::generation_processor(
                    chunks_clone,
                    generation_channel_receiver,
                    mesh_sender_clone,
                );
            });
        }

        for _i in 0..2 {
            let chunks_clone = Arc::clone(&self.chunks);
            let generation_pre_processor_receiver = self.generation_pre_processor_channel.1.clone();
            let initialization_queue_clone = Arc::clone(&self.initialization_queue);
            let initialization_sender = self.initialization_channel.0.clone();
            let generation_sender_clone = self.generation_channel.0.clone();
            self.thread_pool.spawn(move || {
                VoxelScene::generation_pre_processor(
                    chunks_clone,
                    generation_pre_processor_receiver,
                    initialization_queue_clone,
                    initialization_sender,
                    generation_sender_clone,
                );
            });
        }

        println!(
            "[INFO] World generation initialized with {} threads",
            self.thread_pool.current_num_threads()
        );
    }

    pub fn initialize_and_generate_chunk(&self, position: IVec3) {
        VoxelScene::request_initialize_chunk(
            Arc::clone(&self.initialization_queue),
            self.initialization_channel.0.clone(),
            (
                position,
                Some(self.generation_pre_processor_channel.0.clone()),
            ),
        );
    }

    pub fn initialization_processor(
        chunks: ChunkMap,
        pos_receiver: Receiver<(IVec3, Option<Sender<IVec3>>)>,
    ) {
        println!("Started initialization processor");
        loop {
            let mut chunks_to_process = pos_receiver.try_iter().collect::<Vec<_>>();
            if chunks_to_process.len() == 0 {
                chunks_to_process = vec![pos_receiver.recv().unwrap()]; // Nothing to process, wait for something
            }
            chunks_to_process.iter().for_each(|(chunk_pos, callback)| {
                if chunks.contains_key(&chunk_pos) {
                    println!("INITIALIZING CHUNK THAT ALREADY EXISTS!");
                    return;
                }
                let mut chunk = VoxelChunk::new(*chunk_pos);

                // Set chunk data
                let base_wavelength = 200.0;

                let chunk_pos_scenespace = chunk.scenespace_pos();
                let (noise, _min, _max) = NoiseBuilder::fbm_3d_offset(
                    chunk_pos_scenespace.x as f32,
                    CHUNK_SIZE as usize,
                    chunk_pos_scenespace.y as f32,
                    CHUNK_SIZE as usize,
                    chunk_pos_scenespace.z as f32,
                    CHUNK_SIZE as usize,
                )
                .with_freq(1.0 / base_wavelength)
                .with_octaves(2)
                .with_lacunarity(5.0)
                .with_gain(0.15)
                .generate();

                let range = 0.025; // fbm produces values up to ~0.02, or 1/50th of a block but as it has additive octaves, the value needs to be slightly larger
                let height_blend = 40.0;
                let avg_block_step_density = range / height_blend;

                chunk
                    .voxels
                    .iter_mut()
                    .enumerate()
                    .for_each(|(index, voxel)| {
                        let voxel_pos = index_to_pos(index as u32);
                        let density = noise
                            .get(pos_to_index_inverse(&voxel_pos) as usize)
                            .unwrap()
                            - ((voxel_pos.y as i32 + chunk_pos_scenespace.y) as f32
                                * (range / height_blend))
                            + range;
                        if density > 0.0 {
                            // == The below data is to be used to construct the current voxel ==
                            // Vertical depth
                            // Current slope
                            // Altitude
                            // Density
                            // Moisture level 

                            // NOTE: Perhaps restructure the generation to build top to bottom, so that we can keep track of the current vertical depth

                            chunk.is_empty = false;
                            voxel.shape = voxel_shape::CUBE;
                            if density > avg_block_step_density {
                                voxel.id = 2;
                            } else {
                                voxel.id = 1;
                            }
                        }
                    });

                chunks.insert(*chunk_pos, chunk);
                callback.as_ref().map(|s| s.send(*chunk_pos));
            });
        }
    }

    pub fn generation_processor(
        chunks: ChunkMap,
        pos_receiver: Receiver<IVec3>,
        mesh_sender: Sender<(IVec3, Mesh)>,
    ) {
        println!("Started generation processor");
        loop {
            let chunk_pos = pos_receiver.recv().unwrap();
            let chunk = (*chunks.get(&chunk_pos).unwrap()).clone();
            let chunks_clone = Arc::clone(&chunks);
            let mesh = chunk.generate_mesh(chunks_clone);
            mesh_sender.send((chunk_pos, mesh)).unwrap();
        }
    }

    pub fn generation_pre_processor(
        chunks: ChunkMap,
        pos_receiver: Receiver<IVec3>,
        initialization_queue: Arc<DashSet<IVec3>>,
        initialization_sender: Sender<(IVec3, Option<Sender<IVec3>>)>,
        pos_sender: Sender<IVec3>,
    ) {
        println!("Started generation pre-processor");
        // store a list of chunk positions
        let mut chunks_to_generate = VecDeque::new();
        loop {
            let mut chunk_positions = pos_receiver.try_iter().collect::<Vec<_>>();
            chunk_positions.extend(chunks_to_generate.iter());
            chunks_to_generate.clear();
            if chunk_positions.len() == 0 {
                chunk_positions = vec![pos_receiver.recv().unwrap()]; // Nothing left in queue, wait for something
            }
            for chunk_pos in chunk_positions {
                // get a list of neighbours
                let mut failed = false;
                for direction in voxel_directions::ALL {
                    let neighbour_pos = chunk_pos + direction.as_vec();
                    if !chunks.contains_key(&neighbour_pos) {
                        failed = true;
                        VoxelScene::request_initialize_chunk(
                            initialization_queue.clone(),
                            initialization_sender.clone(),
                            (neighbour_pos, None),
                        );
                    }
                }

                // if all neighbours are initialized, schedule the chunk to be generated
                if !failed && chunks.contains_key(&chunk_pos) {
                    if !chunks.get(&chunk_pos).unwrap().is_empty {
                        pos_sender.send(chunk_pos).unwrap();
                    }
                } else {
                    chunks_to_generate.push_front(chunk_pos);
                }
            }
        }
    }
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
                    shape: voxel_shape::CUBE,
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

    pub fn generate_mesh(&self, scene_chunks: ChunkMap) -> Mesh {
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

        mesh.append_vertices(&mut vertices);
        mesh.append_indices(&mut indices);

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

pub fn pos_to_index_inverse(pos: &UVec3) -> u32 {
    (pos.z * CHUNK_SIZE * CHUNK_SIZE) + (pos.y * CHUNK_SIZE) + pos.x
}

#[inline(always)]
fn generate_faces(
    voxel: &VoxelData,
    scene_chunks: ChunkMap,
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

    let color = voxel_registry::get_voxel_by_id(voxel.id)
        .unwrap()
        .color
        .into();
    let mut append_mesh = |mesh: &Mesh| {
        let index_offset = vertices.len() as u32;

        let flip_x = voxel.shape.extract_flip_x();
        let flip_y = voxel.shape.extract_flip_y();
        let flip_z = voxel.shape.extract_flip_z();
        let flip_count = (flip_x as u32 + flip_y as u32 + flip_z as u32) % 2;

        if flip_count & 1 == 0 {
            let mut new_indices = mesh.get_indices().clone();
            new_indices
                .iter_mut()
                .for_each(|index| *index += index_offset);
            indices.append(&mut new_indices);
        } else {
            indices.reserve(mesh.index_count);
            let mut new_indices = mesh.get_indices().clone();
            new_indices
                .iter_mut()
                .for_each(|index| *index += index_offset);
            new_indices.reverse();
            indices.append(&mut new_indices);
        }

        vertices.reserve(mesh.vertex_count);

        mesh.get_vertices().iter().for_each(|v| {
            let mut vert = v.clone();
            vert.color = color;
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

    // TODO: Consider caching orientations?
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
