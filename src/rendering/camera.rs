use crate::state::State;
use glam::{Mat4, Quat, Vec3};
use wgpu::{util::DeviceExt, BindGroup, Buffer};

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub uniform: CameraUniform,
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub render_layers: Vec<String>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_transform_matrix(&self) -> Mat4 {
        let view = Mat4::from_rotation_translation(self.rotation, self.position).inverse();
        view
    }

    pub fn build_projection_matrix(&self) -> Mat4 {
        let proj = Mat4::perspective_lh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
        proj
    }

    pub fn update_uniform(&mut self) {
        self.uniform.projection = self.build_projection_matrix().to_cols_array_2d();
        self.uniform.transform = self.build_transform_matrix().to_cols_array_2d();
    }

    pub fn new(state: &State) -> Camera {
        let uniform = CameraUniform::new();

        let buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &state.camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_passes = Vec::new();

        let aspect = state.size.width as f32 / state.size.height as f32;
        let fovy = 50.0;
        let znear = 0.01;
        let zfar = 2000.0;

        let position = Vec3::ZERO;
        let rotation = Quat::IDENTITY;

        let mut cam = Camera {
            position,
            rotation,
            uniform,
            buffer,
            bind_group,
            render_layers: render_passes,
            aspect,
            fovy,
            znear,
            zfar,
        };
        cam.update_uniform();
        cam
    }

    pub fn add_render_layer(&mut self, layer_name: String) {
        self.render_layers.push(layer_name);
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
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
}
