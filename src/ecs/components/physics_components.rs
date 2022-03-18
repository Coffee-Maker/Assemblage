pub mod collider_components {
    use crate::{asset_types::mesh::Mesh, next_id, physics::physics_scene::PhysicsScene};
    use parking_lot::RwLock;
    use rapier3d::{
        math::Point,
        prelude::{Collider, ColliderBuilder, ColliderHandle, Shape, SharedShape},
    };
    use std::sync::Arc;

    pub trait ColliderComponent {
        fn get_handle(&self) -> ColliderHandle;
    }

    pub struct MeshCollider {
        scene: Arc<RwLock<PhysicsScene>>,
        mesh: Arc<RwLock<Mesh>>,
        handle: ColliderHandle,
    }

    impl MeshCollider {
        pub fn new(mesh: Arc<RwLock<Mesh>>, scene: Arc<RwLock<PhysicsScene>>) -> Self {
            let mesh_lock = mesh.read();
            let mesh_indices = mesh_lock.get_indices();
            let mut indices = Vec::new();
            for i in 0..mesh_lock.index_count / 3 {
                indices.push([
                    *mesh_indices.get(i * 3).unwrap(),
                    *mesh_indices.get((i * 3) + 1).unwrap(),
                    *mesh_indices.get((i * 3) + 2).unwrap(),
                ]);
            }

            let collider = ColliderBuilder::trimesh(
                mesh_lock
                    .get_vertices()
                    .iter()
                    .map(|v| Point::from(v.position))
                    .collect(),
                indices,
            )
            .build();
            Self {
                scene: Arc::clone(&scene),
                mesh: Arc::clone(&mesh),
                handle: scene.write().register_collider(collider),
            }
        }
    }

    impl ColliderComponent for MeshCollider {
        fn get_handle(&self) -> ColliderHandle {
            self.handle
        }
    }
}

pub mod body_components {
    use std::sync::Arc;

    use cgmath::EuclideanSpace;
    use glam::{Quat, Vec3, Vec4Swizzles};
    use parking_lot::RwLock;
    use rapier3d::{
        na::Translation3,
        prelude::{Collider, RigidBody, RigidBodyBuilder, RigidBodyHandle},
    };

    use crate::{next_id, physics::physics_scene::PhysicsScene};

    pub struct DynamicBody {
        scene: Arc<RwLock<PhysicsScene>>,
        collider: Collider,
        handle: RigidBodyHandle,
    }

    impl DynamicBody {
        pub fn new(collider: Collider, scene: Arc<RwLock<PhysicsScene>>) -> Self {
            let rb = RigidBodyBuilder::new(rapier3d::prelude::RigidBodyType::Dynamic).build();
            Self {
                scene: Arc::clone(&scene),
                collider,
                handle: scene.write().register_rigidbody(rb),
            }
        }

        pub fn set_gravity_scale(&mut self, gravity_scale: f32) {
            let mut scene_lock = self.scene.write();
            let rb = scene_lock.rigidbodies.get_mut(self.handle).unwrap();
            rb.set_gravity_scale(gravity_scale, true);
        }

        pub fn apply_force(&mut self, force: Vec3) {
            let mut scene_lock = self.scene.write();
            let rb = scene_lock.rigidbodies.get_mut(self.handle).unwrap();
            rb.apply_force(force.to_array().into(), true);
        }

        pub fn get_transform(&self) -> (Vec3, Quat) {
            let mut scene_lock = self.scene.write();
            let rb = scene_lock.rigidbodies.get_mut(self.handle).unwrap();
            let na_position = rb.position();
            let (position, rotation): (Vec3, Quat) = na_position.into(); // PLEASE FIX THIS
        }
    }
}
