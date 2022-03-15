use glam::Vec3;
use rapier3d::prelude::*;

pub struct PhysicsScene {
    rigidbodies: RigidBodySet,
    colliders: ColliderSet,
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
    pub fn new(update_rate: u32) -> PhysicsScene {
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

    fn step_scene(&mut self) {
        for _ in 0..200 {
            self.physics_pipeline.step(
                &vector![self.gravity.x, self.gravity.y, self.gravity.z],
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

    fn physics_scene_processor() {
        println!("Started physics scene processor");
        loop {
            // Fixed update loop
        }
        /* Create the bounding ball. */
        // let rigid_body = RigidBodyBuilder::new_dynamic()
        //     .translation(vector![0.0, 10.0, 0.0])
        //     .build();
    }
}
