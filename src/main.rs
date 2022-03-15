#![feature(int_roundings)]

mod asset_providers;
mod ecs;
mod input_manager;
mod physics;
mod rendering;
mod state;
mod time;
mod voxels;

use ecs::{
    components::{
        self,
        camera::Camera,
        player_components::Player,
        rendering_components::MeshRenderer,
        transformation_components::{Position, Rotation},
    },
    systems::{
        camera_systems::update_camera_system, player_controller::update_players_system,
        render_systems::construct_buffers,
    },
    world::World,
};
use input_manager::update_inputs;
use legion::IntoQuery;
use legion::{Resources, Schedule};
use mimalloc::MiMalloc;
use parking_lot::RwLock;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rendering::{
    material::{Material, MaterialDiffuseTexture},
    render_pass_data::{create_render_pass, render_layers},
    texture::Texture,
};
use state::*;
use std::{collections::HashMap, sync::Arc, time::Instant};
use time::Time;
use voxels::voxel_scene::CHUNK_SIZE;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[macro_use]
extern crate lazy_static;
extern crate nalgebra as na;

use glam::{IVec3, Quat, UVec3, Vec3};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use crate::rendering::mesh::Mesh;

use crate::{rendering::mesh::Mesh, voxels::voxel_scene::VoxelScene};

fn main() -> Result<(), ()> {
    env_logger::init(); // Tells WGPU to inform us of errors, rather than failing silently

    let event_loop = EventLoop::new();
    // Create a window
    let window = WindowBuilder::new()
        .with_maximized(true)
        .build(&event_loop)
        .unwrap();

    let state = Arc::new(RwLock::new(State::new(&window).await));

    // Setup entity world
    let state_clone = Arc::clone(&state);
    // Create a Legion world (ECS)
    let world = Arc::new(RwLock::new(World {
        legion_world: legion::World::default(),
    }));

    let state_lock = state_clone.write();
    let camera = Arc::new(RwLock::new(rendering::camera::Camera::new(&state_lock)));

    let diffuse_bytes = include_bytes!("textures/lapis_block.png");
    let texture = Arc::new(
        Texture::from_bytes(
            &state_lock.device,
            &state_lock.queue,
            diffuse_bytes,
            "lapis",
        )
        .unwrap(),
    );

    let material: Arc<RwLock<dyn Material>> = Arc::new(RwLock::new(MaterialDiffuseTexture::new(
        &state_lock,
        texture,
    )));

    drop(state_lock);

    // Create the default render layer
    render_layers::create_layer("Default".to_string());

    let world_mesh = Arc::new(RwLock::new(Mesh::new()));

    let mut camera_lock = camera.write();
    camera_lock.add_render_layer("Default".to_string());
    drop(camera_lock);

    let mut world_lock = world.write();
    world_lock.legion_world.push((
        Position(Vec3::ZERO),
        Rotation(Quat::IDENTITY),
        Player { fly_speed: 50.0 },
        components::camera::Camera { camera },
    ));
    world_lock.legion_world.push((
        Position(Vec3::ZERO),
        Rotation(Quat::IDENTITY),
        MeshRenderer {
            mesh: Arc::clone(&world_mesh),
            material: Arc::clone(&material),
            render_layer: "Default".to_string(),
        },
    ));
    drop(world_lock);

    let world_clone = Arc::clone(&world);
    rayon::spawn(move || {
        // Add systems
        let mut schedule = Schedule::builder()
            .add_system(update_players_system())
            .add_system(update_camera_system())
            .build();
        let start = Instant::now();
        let mut loop_time = Instant::now();
        let mut resources = Resources::default(); // Resources are accessible to all systems that use them
        loop {
            update_inputs(); // Update the inputs before sending firing the systems
            resources.insert(Time {
                time: start.elapsed().as_secs_f64(),
                delta_time: loop_time.elapsed().as_secs_f64(),
            });
            loop_time = Instant::now();

            let mut world_lock = world_clone.write();
            schedule.execute(&mut world_lock.legion_world, &mut resources);
        }
    });

    // Setup voxel scene
    let mut scene = VoxelScene::new();
    generate_world(&mut scene, Arc::clone(&world_mesh), UVec3::new(50, 5, 50));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                let mut state_lock = state.write();
                if !state_lock.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state_lock.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state_lock.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let world_lock = world.read();
                let mut query = <&Camera>::query();

                let cameras: Vec<Arc<RwLock<rendering::camera::Camera>>> = query
                    .iter(&world_lock.legion_world)
                    .map(|cam| Arc::clone(&cam.camera))
                    .collect();

                let mut state_lock = state.write();
                construct_buffers(&state_lock, &world_lock.legion_world);

                match state_lock.render(cameras) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => {
                        let size = state_lock.size;
                        state_lock.resize(size);
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}

pub fn generate_world(scene: &mut VoxelScene, world_mesh: Arc<RwLock<Mesh>>, size: UVec3) {
    for x in 0..size.x {
        for y in 0..size.y {
            for z in 0..size.z {
                scene.initialize_and_generate_chunk(IVec3::new(x as i32, y as i32, z as i32));
            }
        }
    }

    let (tx, rx) = flume::unbounded();
    scene.setup_chunk_processors(tx);
  
    rayon::spawn(move || {
        let mut saved_meshes = HashMap::new();
        loop {
            let mut regenerate = false;
            rx.try_iter().for_each(|(k, v)| {
                saved_meshes.insert(k, v);
                regenerate = true;
            });
            if !regenerate {
                // Nothing to process, wait for something
                let (k, v) = rx.recv().unwrap();
                saved_meshes.insert(k, v);
            }

            let meshes = saved_meshes
                .par_iter_mut()
                .map(|(position, chunk)| {
                    let mut mesh = chunk.clone();

                    mesh.vertices.iter_mut().for_each(|vert| {
                        vert.position = [
                            vert.position[0] + (position.x as f32 * CHUNK_SIZE as f32),
                            vert.position[1] + (position.y as f32 * CHUNK_SIZE as f32),
                            vert.position[2] + (position.z as f32 * CHUNK_SIZE as f32),
                        ]
                    });

                    mesh
                })
                .collect::<Vec<Mesh>>();
            let mut combined_verts = Vec::new();
            let mut combined_indices = Vec::new();
            meshes
                .into_iter()
                .map(|mesh| (mesh.vertices, mesh.indices))
                .for_each(|(mut verts, indics)| {
                    let offset = combined_verts.len() as u32;

                    combined_verts.append(&mut verts);

                    combined_indices.reserve(indics.len());
                    combined_indices.extend(indics.iter().map(|&x| x + offset));
                });

            let mut world_mesh_lock = world_mesh.write();
            world_mesh_lock.vertices = combined_verts;
            world_mesh_lock.indices = combined_indices;
        }
    });
}
