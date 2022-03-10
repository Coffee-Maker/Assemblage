use glam::{Quat, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(pub Vec3);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotation(pub Quat);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale(pub Vec3);
