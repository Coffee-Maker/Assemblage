use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use flume::{Receiver, Sender};
use glam::{IVec3, UVec3};
use noise::{NoiseFn, Perlin};

use crate::rendering::mesh::Mesh;
use crate::rendering::vertex::Vertex;
use crate::voxels::voxel_data::VoxelData;
use crate::voxels::voxel_shapes::voxel_shapes;

use super::voxel_mesh::get_voxel_mesh;
use super::voxel_shapes::{voxel_directions, VoxelDirection, VoxelShape};

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
        chunk_lock
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
        for _i in 0..4 {
            VoxelScene::initialization_processor(
                self.initialization_channel.1.clone(),
                self.registration_channel.0.clone(),
            );
        }

        for _i in 0..4 {
            let chunks_clone = Arc::clone(&self.chunks);
            VoxelScene::generation_processor(
                chunks_clone,
                self.generation_channel.1.clone(),
                mesh_sender.clone(),
            );
        }

        for _ in 0..1 {
            let chunks_clone = Arc::clone(&self.chunks);
            VoxelScene::registration_processor(
                self.registration_channel.1.clone(),
                chunks_clone,
                self.generation_channel.0.clone(),
            );
        }
    }

    pub fn initialize_chunk(&self, position: IVec3) {
        self.initialization_channel.0.send(position).unwrap();
    }

    pub fn initialization_processor(
        pos_receiver: Receiver<IVec3>,
        chunk_sender: Sender<VoxelChunk>,
    ) {
        rayon::spawn(move || {
            println!("Started initialization processor");

            loop {
                let chunk_pos = pos_receiver.recv().unwrap();

                let mut chunk = VoxelChunk::new(chunk_pos);

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
                                voxel_shapes::CORNER_STAIR
                            } else {
                                voxel_shapes::CUBE
                            };
                            voxel.id = 1;
                        }
                    });

                chunk_sender.send(chunk).unwrap();
            }
        });
    }

    pub fn registration_processor(
        chunk_receiver: Receiver<VoxelChunk>,
        chunks: Arc<Mutex<HashMap<IVec3, VoxelChunk>>>,
        pos_sender: Sender<IVec3>,
    ) {
        rayon::spawn(move || loop {
            let chunk = chunk_receiver.recv().unwrap();
            let pos = chunk.position;
            let mut chunks_lock = chunks.lock().unwrap();
            chunks_lock.insert(pos, chunk);
            pos_sender.send(pos).unwrap();
        });
    }

    pub fn generation_processor(
        chunks: Arc<Mutex<HashMap<IVec3, VoxelChunk>>>,
        pos_receiver: Receiver<IVec3>,
        mesh_sender: Sender<(IVec3, Mesh)>,
    ) {
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
    let height_offset: f32 = 0.0;
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

    pub fn generate_mesh(&self, scene_chunks: Arc<Mutex<HashMap<IVec3, VoxelChunk>>>) -> Mesh {
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
    scene_chunks: Arc<Mutex<HashMap<IVec3, VoxelChunk>>>,
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
        let scene_chunks_lock = scene_chunks.lock().unwrap();
        let neighbour = chunk.voxel_scenespace_at(&sample_position).map_or_else(
            || {
                scene_chunks_lock
                    .get(&VoxelScene::chunk_at(&sample_position))
                    .map_or(None, |chunk| chunk.voxel_scenespace_at(&sample_position))
            },
            |voxel| Some(voxel),
        );
        neighbour.map_or(true, |neighbour| {
            //println!("Self {:#08b}", VoxelShape::get_face_shape(voxel.shape, direction));
            //println!("Neighbour {:#08b}", VoxelShape::get_face_shape(neighbour.shape, direction.flip()));
            neighbour.id == 0
                || !neighbour
                    .shape
                    .face_contains(direction.flip(), (voxel.shape, direction))
        })
    };

    let mut append_mesh = |mesh: &Mesh| {
        let index_offset = vertices.len();
        indices.reserve(mesh.indices.len());
        mesh.indices
            .iter()
            .for_each(|i| indices.push(i + index_offset as u32));

        vertices.reserve(mesh.vertices.len());
        mesh.vertices.iter().for_each(|v| {
            let mut vert = v.clone();
            let x = v.position[0]
                * if voxel.shape.extract_flip_x() {
                    -1.0
                } else {
                    1.0
                };
            let y = v.position[1]
                * if voxel.shape.extract_flip_y() {
                    -1.0
                } else {
                    1.0
                };
            let z = v.position[2]
                * if voxel.shape.extract_flip_z() {
                    -1.0
                } else {
                    1.0
                };
            let (y, z) = if voxel.shape.extract_rotate_x() {
                (-z, y)
            } else {
                (y, z)
            };
            let (x, y) = if voxel.shape.extract_rotate_z() {
                (y, -x)
            } else {
                (x, y)
            };
            vert.position[0] = x + f_position.x;
            vert.position[1] = y + f_position.y;
            vert.position[2] = z + f_position.z;
            vertices.push(vert);
        });
    };

    let shape_mesh = get_voxel_mesh(voxel.shape);

    append_mesh(&shape_mesh.always);

    // North
    if face_check(voxel_directions::NORTH) {
        append_mesh(&shape_mesh.north);
    }

    // South
    if face_check(voxel_directions::SOUTH) {
        append_mesh(&shape_mesh.south);
    }

    // East
    if face_check(voxel_directions::EAST) {
        append_mesh(&shape_mesh.east);
    }

    // West
    if face_check(voxel_directions::WEST) {
        append_mesh(&shape_mesh.west);
    }

    // Up
    if face_check(voxel_directions::UP) {
        append_mesh(&shape_mesh.top);
    }

    // Down
    if face_check(voxel_directions::DOWN) {
        append_mesh(&shape_mesh.bottom);
    }
}
