use std::{fmt::Debug, sync::Arc};
use wgpu::{BindGroup, BindGroupLayout, PrimitiveTopology, RenderPipeline, ShaderModule};

use crate::state::State;

use super::{
    texture::{self, Texture},
    vertex::Vertex,
};

pub trait Material: Debug + Sync + Send {
    fn get_pipeline(&self, state: &State) -> Arc<RenderPipeline>;
    fn get_texture_bind_group(&self, state: &State) -> Arc<BindGroup>;
    fn get_texture_bind_group_layout(&self, state: &State) -> Arc<BindGroupLayout>;
    fn get_shader(&self, state: &State) -> Arc<ShaderModule>;
}

// Structs for the various kinds of materials
#[derive(Debug)]
pub struct MaterialDiffuseTexture {
    pub diffuse_texture: Arc<Texture>,
    texture_bind_group: Option<Arc<BindGroup>>,
    pipeline: Option<Arc<RenderPipeline>>,
}

impl MaterialDiffuseTexture {
    pub fn new(state: &State, diffuse_texture: Arc<Texture>) -> MaterialDiffuseTexture {
        MaterialDiffuseTexture {
            diffuse_texture,
            texture_bind_group: None,
            pipeline: None,
        }
    }
}

impl Material for MaterialDiffuseTexture {
    fn get_pipeline(&self, state: &State) -> Arc<RenderPipeline> {
        // TODO: Cache the pipeline in PIPELINES
        Arc::new(create_pipeline(
            state,
            self.get_texture_bind_group_layout(state),
            self.get_shader(state),
        ))
    }

    fn get_texture_bind_group(&self, state: &State) -> Arc<BindGroup> {
        Arc::new(state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.get_texture_bind_group_layout(state),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        }))
    }

    fn get_texture_bind_group_layout(&self, state: &State) -> Arc<BindGroupLayout> {
        Arc::new(
            state
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
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                }),
        )
    }

    fn get_shader(&self, state: &State) -> Arc<ShaderModule> {
        Arc::new(
            state
                .device
                .create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some("Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
                }),
        )
    }
}

// Create a render pipeline
pub fn create_pipeline(
    state: &State,
    texture_bind_group_layout: Arc<BindGroupLayout>,
    shader: Arc<ShaderModule>,
) -> RenderPipeline {
    let render_pipeline_layout =
        state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &state.camera_bind_group_layout],
                push_constant_ranges: &[],
            });

    state
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
                    format: state.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
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
        })
}
