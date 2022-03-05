#![feature(int_roundings)]

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[macro_use]
extern crate lazy_static;

mod camera_controller;
mod rendering;
mod state;
mod voxels;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use state::*;
use voxels::voxel_scene::CHUNK_SIZE;

use glam::{IVec3, UVec3};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{rendering::mesh::Mesh, voxels::voxel_scene::VoxelScene};

#[tokio::main]
async fn main() -> Result<(), ()> {
    env_logger::init(); // Tells WGPU to inform us of errors, rather than failing silently

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap(); // Create a window
    let mut scene = VoxelScene::new();
    let state = Arc::new(Mutex::new(State::new(&window).await));

    let state_clone = Arc::clone(&state);
    generate_world(&mut scene, state_clone, UVec3::new(25, 5, 25));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                let mut state_lock = state.lock().unwrap();
                if !state_lock.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
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
                let mut state_lock = state.lock().unwrap();
                state_lock.update();
                match state_lock.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => {
                        let size = state_lock.size;
                        state_lock.resize(size)
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

pub fn generate_world(scene: &mut VoxelScene, state: Arc<Mutex<State>>, size: UVec3) {
    let mut state_lock = state.lock().unwrap();
    state_lock.render_passes.clear();
    state_lock.add_render_pass();

    for x in 0..size.x {
        for y in 0..size.y {
            for z in 0..size.z {
                scene.initialize_chunk(IVec3::new(x as i32, y as i32, z as i32));
            }
        }
    }

    let (tx, rx) = flume::unbounded();
    scene.setup_chunk_processors(tx);

    // Regenerate mesh when a mesh is ready for submission
    let state_clone = Arc::clone(&state);
    rayon::spawn(move || {
        let mut saved_meshes = HashMap::new();
        loop {
            let mut count = 0;
            loop {
                let data = rx.try_recv();
                match data {
                    Ok(data) => {
                        saved_meshes.insert(data.0, data.1);
                    }
                    Err(_) => break,
                }
                if count > 300 {
                    break;
                }
                count += 1;
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

            let vert_count = combined_verts.len();
            let mut state_lock = state_clone.lock().unwrap();

            let pass_index = state_lock.render_passes.len() - 1;
            let mut pass = state_lock.render_passes.remove(pass_index);

            pass.set_vertices(&state_lock.device, &mut combined_verts);
            pass.set_indices(&state_lock.device, &mut combined_indices);

            state_lock.render_passes.push(pass);

            if vert_count > 100000 {
                state_lock.add_render_pass();
                saved_meshes.clear();
            }
        }
    });
}
