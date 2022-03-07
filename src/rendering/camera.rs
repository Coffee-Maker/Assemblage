#[rustfmt::skip]

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

use glam::{Mat4, Vec3};

impl Camera {
    pub fn build_transform_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_lh(self.eye, self.target, self.up);
        view
    }

    pub fn build_projection_matrix(&self) -> Mat4 {
        let proj = Mat4::perspective_lh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
        proj
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    projection: [[f32; 4]; 4],
    transform: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            projection: Mat4::IDENTITY.to_cols_array_2d(),
            transform: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.projection = camera.build_projection_matrix().to_cols_array_2d();
        self.transform = camera.build_transform_matrix().to_cols_array_2d();
    }
}
