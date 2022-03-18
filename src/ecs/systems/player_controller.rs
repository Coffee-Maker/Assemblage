use glam::{Quat, Vec3};
use legion::system;
use winit::event::{MouseButton, VirtualKeyCode};

use crate::{
    components::{
        player_components::Player,
        transformation_components::{Position, Rotation},
    },
    ecs::components::physics_components::body_components::DynamicBody,
    input_manager::{self, get_mouse_delta},
    time::Time,
};

#[system(for_each)]
pub fn update_players(
    pos: &mut Position,
    rot: &mut Rotation,
    player: &Player,
    body: &mut DynamicBody,
    #[resource] time: &Time,
) {
    let mut forward: Vec3 = rot.0.mul_vec3(Vec3::Z).into();
    forward = (forward * Vec3::new(1.0, 0.0, 1.0)).normalize();
    let right: Vec3 = rot.0.mul_vec3(Vec3::X).into();
    let up: Vec3 = Vec3::Y;

    if input_manager::get_key(VirtualKeyCode::W) {
        body.apply_force(forward * time.delta_time as f32 * player.fly_speed);
    }

    if input_manager::get_key(VirtualKeyCode::S) {
        body.apply_force(-1.0 * forward * time.delta_time as f32 * player.fly_speed);
    }

    if input_manager::get_key(VirtualKeyCode::D) {
        body.apply_force(right * time.delta_time as f32 * player.fly_speed);
    }

    if input_manager::get_key(VirtualKeyCode::A) {
        body.apply_force(-1.0 * right * time.delta_time as f32 * player.fly_speed);
    }

    if input_manager::get_key(VirtualKeyCode::Space) {
        body.apply_force(up * time.delta_time as f32 * player.fly_speed);
    }

    if input_manager::get_key(VirtualKeyCode::LShift) {
        body.apply_force(-1.0 * up * time.delta_time as f32 * player.fly_speed);
    }

    if input_manager::get_button(MouseButton::Right) {
        let delta = get_mouse_delta() * 0.003;
        rot.0 = Quat::from_axis_angle(right, delta.y) * rot.0;
        rot.0 = Quat::from_axis_angle(up, delta.x) * rot.0;
    }

    pos.0 = body.get_transform().0;
}
