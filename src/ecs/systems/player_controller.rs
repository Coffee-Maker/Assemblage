use glam::{Quat, Vec3};
use legion::system;
use winit::event::{MouseButton, VirtualKeyCode};

use crate::{
    components::{
        player_components::Player,
        transformation_components::{Position, Rotation},
    },
    input_manager::{self, get_mouse_delta},
    time::Time,
};

#[system(for_each)]
pub fn update_players(
    pos: &mut Position,
    rot: &mut Rotation,
    player: &Player,
    #[resource] time: &Time,
) {
    let mut forward: Vec3 = rot.0.mul_vec3(Vec3::Z).into();
    forward = (forward * Vec3::new(1.0, 0.0, 1.0)).normalize();
    let right: Vec3 = rot.0.mul_vec3(Vec3::X).into();
    let up: Vec3 = Vec3::Y;

    if input_manager::get_key(VirtualKeyCode::W) {
        pos.0 += forward * time.delta_time as f32 * player.fly_speed;
    }

    if input_manager::get_key(VirtualKeyCode::S) {
        pos.0 -= forward * time.delta_time as f32 * player.fly_speed;
    }

    if input_manager::get_key(VirtualKeyCode::D) {
        pos.0 += right * time.delta_time as f32 * player.fly_speed;
    }

    if input_manager::get_key(VirtualKeyCode::A) {
        pos.0 -= right * time.delta_time as f32 * player.fly_speed;
    }

    if input_manager::get_key(VirtualKeyCode::Space) {
        pos.0 += up * time.delta_time as f32 * player.fly_speed;
    }

    if input_manager::get_key(VirtualKeyCode::LShift) {
        pos.0 -= up * time.delta_time as f32 * player.fly_speed;
    }

    if input_manager::get_button(MouseButton::Right) {
        let delta = get_mouse_delta() * 0.003;
        rot.0 = Quat::from_axis_angle(right, delta.y) * rot.0;
        rot.0 = Quat::from_axis_angle(up, delta.x) * rot.0;
    }
}
