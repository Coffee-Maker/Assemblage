use crate::components::{
    camera::Camera,
    player_components::Player,
    transformation_components::{Position, Rotation},
};

pub type PlayerEntity = (Position, Rotation, Player);
pub type CameraEntity = (Position, Rotation, Camera);
