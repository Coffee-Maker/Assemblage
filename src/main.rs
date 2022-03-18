#![feature(int_roundings)]

mod asset_types;
mod ecs;
mod input_manager;
mod noise;
mod physics;
mod rendering;
mod state;
mod time;
mod voxels;

use ecs::{
    components::{
        self,
        camera::Camera,
        physics_components::{body_components::DynamicBody, collider_components::MeshCollider},
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
use physics::physics_scene::PhysicsScene;
use pollster::block_on;
use rapier3d::prelude::ColliderBuilder;
use rendering::{
    material::{Material, MaterialDiffuseTexture},
    render_pass_data::render_layers,
    texture::Texture,
};
use state::*;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};
use time::Time;
use voxels::voxel_scene::CHUNK_SIZE;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[macro_use]
extern crate lazy_static;
extern crate nalgebra as na;

use glam::{EulerRot, IVec3, Quat, UVec3, Vec3};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::voxels::voxel_scene::VoxelScene;

fn main() {
    env_logger::init(); // Tells WGPU to inform us of errors, rather than failing silently

    let event_loop = EventLoop::new();
    // Create a window
    let window = WindowBuilder::new()
        .with_maximized(true)
        .build(&event_loop)
        .unwrap();

    let state = Arc::new(RwLock::new(block_on(State::new(&window))));

    // Setup entity world
    let state_clone = Arc::clone(&state);
    // Create a Legion world (ECS)
    let world = Arc::new(RwLock::new(World {
        legion_world: legion::World::default(),
    }));
    let physics_scene = Arc::new(RwLock::new(PhysicsScene::new()));

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

    let mut camera_lock = camera.write();
    camera_lock.add_render_layer("Default".to_string());
    drop(camera_lock);

    let mut world_lock = world.write();
    world_lock.legion_world.push((
        Position(Vec3::new(0.0, 80.0, 0.0)), // Middle of world
        Rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.0,
            (45.0 as f32).to_radians(),
            0.0,
        )),
        Player { fly_speed: 50.0 },
        components::camera::Camera { camera },
        DynamicBody::new(
            ColliderBuilder::ball(1.0).build(),
            Arc::clone(&physics_scene),
        ),
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
    let scene = Arc::new(RwLock::new(VoxelScene::new()));
    generate_world(
        Arc::clone(&scene),
        Arc::clone(&world),
        Arc::clone(&physics_scene),
        Arc::clone(&material),
        UVec3::new(25, 5, 25),
    );

    //let state_clone = Arc::clone(&state);
    //rayon::spawn(move || {
    //    let state_lock = state_clone.read();
    //    let simplex = Simplex3D::new(&state_lock, UVec3::new(128, 128, 128));
    //    let now = Instant::now();
    //    let noise = block_on(simplex.build_noise(&state_lock));
    //    println!("Obtained noise in {:?}", now.elapsed());
    //    //noise.iter().for_each(|v| println!("{v}"));
    //});

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

pub fn generate_world(
    scene: Arc<RwLock<VoxelScene>>,
    world: Arc<RwLock<World>>,
    physics_scene: Arc<RwLock<PhysicsScene>>,
    material: Arc<RwLock<dyn Material>>,
    size: UVec3,
) {
    for x in 0..size.x {
        for y in 0..size.y {
            for z in 0..size.z {
                scene
                    .write()
                    .initialize_and_generate_chunk(IVec3::new(x as i32, y as i32, z as i32));
            }
        }
    }

    let (tx, rx) = flume::unbounded();
    scene.write().setup_chunk_processors(tx);
    rayon::spawn(move || {
        //let mut saved_meshes = HashMap::new();
        loop {
            let (mesh_pos, mesh) = rx.recv().unwrap();
            let mut world_lock = world.write();
            let mesh = Arc::new(RwLock::new(mesh));
            world_lock.legion_world.push((
                Position(mesh_pos.as_vec3() * CHUNK_SIZE as f32),
                Rotation(Quat::IDENTITY),
                MeshRenderer::new(
                    Arc::clone(&mesh),
                    Arc::clone(&material),
                    "Default".to_string(),
                ),
                MeshCollider::new(Arc::clone(&mesh), Arc::clone(&physics_scene)),
            ));
        }
    });
}

lazy_static! {
    static ref CURRENT_ID: AtomicU64 = AtomicU64::new(0);
}

fn next_id() -> u64 {
    CURRENT_ID.fetch_add(1, Ordering::Relaxed);
    CURRENT_ID.load(Ordering::Relaxed)
}
