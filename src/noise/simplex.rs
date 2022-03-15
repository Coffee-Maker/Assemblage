use glam::UVec3;
use std::{borrow::Cow, time::Instant};

use crate::state::State;

use wgpu::BufferUsages;

pub struct Simplex1D {
    pub values: Vec<f32>,
    pub wavelength: f32,
    pub amplitude: f32,
}

impl Simplex1D {
    pub async fn build_noise(state: &State, domain_size: &UVec3) -> Vec<f32> {
        let now = Instant::now();
        let cs_module = state
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "../shaders/noise_compute.wgsl"
                ))),
            });

        // Gets the size in bytes of the buffer.
        let slice_size = domain_size.x * domain_size.y * domain_size.z * 4;

        let size = slice_size as wgpu::BufferAddress;

        // Instantiates buffer without data.
        let output_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Noise Buffer"),
            size,
            usage: BufferUsages::MAP_READ | BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let compute_pipeline =
            state
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Noise compute pipeline"),
                    layout: None,
                    module: &cs_module,
                    entry_point: "main",
                });

        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: output_buffer.as_entire_binding(),
            }],
        });

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&compute_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.insert_debug_marker("compute noise values");
            cpass.dispatch(domain_size.x, domain_size.y, domain_size.z);
        }

        // Submits command encoder for processing
        state.queue.submit(Some(encoder.finish()));

        println!("Submitted in: {:?}", now.elapsed());

        // Note that we're not calling `.await` here.
        let buffer_slice = output_buffer.slice(..);

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
            output_buffer.unmap();

            println!("Total: {:?}", now.elapsed());
            result
        } else {
            panic!("failed to run noise compute on gpu!")
        }
    }
}
