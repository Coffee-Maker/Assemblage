use legion::system;

use crate::ecs::components::{
    camera::Camera,
    transformation_components::{Position, Rotation},
};

#[system(for_each)]
pub fn update_camera(pos: &Position, rot: &Rotation, camera: &mut Camera) {
    let mut cam_lock = camera.camera.write();
    cam_lock.position = pos.0;
    cam_lock.rotation = rot.0;
    cam_lock.update_uniform();
}
