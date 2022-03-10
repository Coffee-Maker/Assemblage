use legion::system;

use crate::ecs::components::{
    camera::Camera,
    transformation_components::{Position, Rotation},
};

#[system(for_each)]
pub fn update_camera(pos: &Position, rot: &Rotation, camera: &mut Camera) {
    camera.camera.position = pos.0;
    camera.camera.rotation = rot.0;
    camera.camera.update_uniform();
}
