use glam::UVec3;
use std::{borrow::Cow, time::Instant};

use crate::state::State;

use wgpu::{BindGroup, Buffer, BufferUsages, ComputePipeline};

pub struct Simplex3D {
    pub domain_size: UVec3,
    pub wavelength: f32,
    pub amplitude: f32,
    buffer: Buffer,
    pipeline: ComputePipeline,
    bind_group: BindGroup,
}

impl Simplex3D {
    pub fn new(state: &State, chunk_size: UVec3, batch_size: u32) -> Self {
        let cs_module = state
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "../shaders/noise_compute.wgsl"
                ))),
            });

        // Gets the size in bytes of the buffer.
        let slice_size = chunk_size.x * chunk_size.y * chunk_size.z * 4;
        let size = slice_size as wgpu::BufferAddress;

        // Instantiates buffer without data.
        let buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Noise Buffer"),
            size,
            usage: BufferUsages::MAP_READ | BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let pipeline = state
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Noise compute pipeline"),
                layout: None,
                module: &cs_module,
                entry_point: "main",
            });

        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            domain_size: chunk_size,
            wavelength: 0.0,
            amplitude: 0.0,
            buffer,
            pipeline,
            bind_group,
        }
    }

    pub async fn build_noise(&self, state: &State) -> Vec<f32> {
        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.dispatch(self.domain_size.x, self.domain_size.y, self.domain_size.z);
        }

        state.queue.submit(Some(encoder.finish()));

        let buffer_slice = self.buffer.slice(..);

        // Gets the future representing when `staging_buffer` can be read from
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

        state.device.poll(wgpu::Maintain::Wait);
        if let Ok(()) = buffer_future.await {
            // Gets contents of buffer
            let data = buffer_slice.get_mapped_range();
            // Since contents are got in bytes, this converts these bytes back to f32
            let result: Vec<_> = data
                .chunks_exact(4)
                .map(|b| f32::from_ne_bytes(b.try_into().unwrap()))
                .collect();

            drop(data);
            self.buffer.unmap();

            result
        } else {
            panic!("failed to run noise compute on gpu!")
        }
    }
}
