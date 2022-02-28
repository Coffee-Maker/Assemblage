#![feature(int_roundings)]

mod camera_controller;
mod rendering;
mod state;
mod voxels;

use state::*;
use voxels::voxel_scene::CHUNK_SIZE;

use glam::{IVec3, UVec3};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
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
    let mut state = State::new(&window).await;

    generate_world(&mut scene, &mut state, UVec3::new(50, 1, 50)).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
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
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
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

pub async fn generate_world(scene: &mut VoxelScene, state: &mut State, size: UVec3) {
    state.render_passes.clear();
    state.add_render_pass();

    // Start timer
    let total_chunk_count = size.x * size.y * size.z;

    use std::time::Instant;
    let now = Instant::now();

    for x in 0..size.x {
        for y in 0..size.y {
            for z in 0..size.z {
                scene.initialize_chunk(&IVec3::new(x as i32, y as i32, z as i32));
            }
        }
    }

    scene.process_initialization_queue().await;

    let pass = state.render_passes.last_mut().unwrap();

    let meshes = scene
        .chunks
        .par_iter_mut()
        .map(|(position, chunk)| {
            let mesh = &mut chunk.mesh;

            mesh.vertices.iter_mut().for_each(|vert| {
                vert.position = [
                    vert.position[0] + (position.x as f32 * CHUNK_SIZE as f32),
                    vert.position[1] + (position.y as f32 * CHUNK_SIZE as f32),
                    vert.position[2] + (position.z as f32 * CHUNK_SIZE as f32),
                ]
            });

            mesh
        })
        .collect::<Vec<&mut Mesh>>();

    let mut combined_verts = Vec::new();
    let mut combined_indices = Vec::new();
    meshes
        .into_iter()
        .map(|mesh| (&mut mesh.vertices, &mut mesh.indices))
        .for_each(|(verts, indics)| {
            let offset = combined_verts.len() as u32;

            combined_verts.append(verts);

            combined_indices.reserve(indics.len());
            combined_indices.extend(indics.iter().map(|&x| x + offset));
        });

    pass.set_vertices(&state.device, &mut combined_verts);
    pass.set_indices(&state.device, &mut combined_indices);

    // End timer
    let elapsed = now.elapsed();
    println!(
        "Generated {} chunks in {elapsed:.2?}\nGeneration took {:.2?} per chunk\nWhich is {} chunks per second",
        total_chunk_count,
        elapsed / total_chunk_count,
        1.0 / (elapsed / total_chunk_count).as_secs_f32()
    );
}
