#[rustfmt::skip]

pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub uniform: CameraUniform,
    pub buffer: Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
    pub render_passes: Vec<RenderPassData>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

use parking_lot::RwLock;
use std::sync::Arc;

use super::{render_pass_data::RenderPassData, texture, vertex::Vertex};
use crate::state::State;
use glam::{Mat4, Quat, Vec3};
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer};

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

        let bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });

        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
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
            bind_group_layout,
            bind_group,
            render_passes,
            aspect,
            fovy,
            znear,
            zfar,
        };
        cam.update_uniform();
        cam
    }

    pub fn add_render_pass(
        &mut self,
        state: Arc<RwLock<State>>,
        topology_type: wgpu::PrimitiveTopology,
    ) {
        let state_lock = state.read();
        let texture_bind_group_layout =
            state_lock
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(
                                // SamplerBindingType::Comparison is only for TextureSampleType::Depth
                                // SamplerBindingType::Filtering if the sample_type of the texture is:
                                //     TextureSampleType::Float { filterable: true }
                                // Otherwise you'll get an error.
                                wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let diffuse_texture = state_lock.get_texture();

        let diffuse_bind_group = state_lock
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
                label: Some("diffuse_bind_group"),
            });

        let shader = state_lock
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
            });

        let render_pipeline_layout =
            state_lock
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &self.bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            state_lock
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[wgpu::ColorTargetState {
                            format: state_lock.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: topology_type,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw, // <- Polygons are wound counter-clockwise
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: texture::Texture::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

        let vertex_buffer =
            state_lock
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: &[],
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            state_lock
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: &[],
                    usage: wgpu::BufferUsages::INDEX,
                });

        let vertex_count = 0;
        let index_count = 0;

        let pass = RenderPassData {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            vertex_count,
            index_count,
            diffuse_bind_group,
        };

        self.render_passes.push(pass);
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
}
