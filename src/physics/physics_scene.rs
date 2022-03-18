use std::sync::Arc;

use glam::Vec3;
use parking_lot::RwLock;
use rapier3d::prelude::*;

pub struct PhysicsScene {
    pub rigidbodies: RigidBodySet,
    pub colliders: ColliderSet,
    gravity: Vec3,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    joint_set: JointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: (),
}

impl PhysicsScene {
    pub fn new() -> PhysicsScene {
        PhysicsScene {
            rigidbodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            joint_set: JointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }

    pub fn register_rigidbody(&mut self, rigidbody: RigidBody) -> RigidBodyHandle {
        self.rigidbodies.insert(rigidbody)
    }

    pub fn register_collider(&mut self, collider: Collider) -> ColliderHandle {
        self.colliders.insert(collider)
    }

    pub fn step_scene(&mut self) {
        self.physics_pipeline.step(
            &self.gravity.to_array().into(),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigidbodies,
            &mut self.colliders,
            &mut self.joint_set,
            &mut self.ccd_solver,
            &self.physics_hooks,
            &self.event_handler,
        );
    }
}

fn physics_scene_processor(scene: Arc<RwLock<PhysicsScene>>) {
    println!("Started physics scene processor");
    loop {
        // TODO: Fixed update loop
        scene.write().step_scene();
    }
}
