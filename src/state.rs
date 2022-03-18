use std::sync::Arc;

use crate::input_manager::set_key;
use crate::input_manager::set_mouse_button;
use crate::input_manager::set_mouse_pos;
use crate::input_manager::PressState;
use crate::rendering::camera::Camera;
use crate::rendering::render_pass_data::render_layers;
use crate::rendering::texture;
use parking_lot::RwLock;
use wgpu::BindGroupLayout;
use wgpu::RenderPassDepthStencilAttachment;
use winit::event::ElementState;
use winit::event::KeyboardInput;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub depth_texture: texture::Texture,
    pub camera_bind_group_layout: BindGroupLayout,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .enumerate_adapters(wgpu::Backends::all())
            .filter(|adapter| {
                // Check if this adapter supports our surface
                surface.get_preferred_format(&adapter).is_some()
            })
            .next()
            .unwrap(); // Finds a suitable adapter

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        surface.configure(&device, &config);

        // Load surface texture
        surface.configure(&device, &config);

        // Depth texture
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        Self {
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,
            camera_bind_group_layout,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            // If the size is < 0 then wgpu is prone to crashing
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            //self.camera.aspect = self.config.width as f32 / self.config.height as f32;

            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                set_key(
                    *keycode,
                    if is_pressed {
                        PressState::Pressed
                    } else {
                        PressState::Released
                    },
                );
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                set_mouse_pos(position);
                true
            }
            WindowEvent::MouseInput { state, button, .. } => {
                set_mouse_button(
                    button,
                    if *state == ElementState::Pressed {
                        PressState::Pressed
                    } else {
                        PressState::Released
                    },
                );
                true
            }
            _ => false,
        }
    }

    pub fn render(&mut self, cameras: Vec<Arc<RwLock<Camera>>>) -> Result<(), wgpu::SurfaceError> {
        for camera in &cameras {
            // Write the camera uniform into the buffer
            let camera_lock = camera.read();
            self.queue.write_buffer(
                &camera_lock.buffer,
                0,
                bytemuck::cast_slice(&[camera_lock.uniform]),
            );

            let output = self.surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            // Create a clear pass
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                }); // The encoder is responsible for sending commands to the GPU via a command buffer.
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what [[location(0)]] in the fragment shader targets
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.3,
                                g: 0.4,
                                b: 0.6,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // Check if the camera has anything to draw before trying to draw
            if camera_lock.render_layers.len() == 0 {
                self.queue.submit(std::iter::once(encoder.finish()));
                output.present();
                continue;
            }
            let mut has_passes = true;
            for layer in render_layers::RENDER_LAYERS.iter() {
                let layer_lock = layer.read();
                if layer_lock.passes.len() == 0 {
                    has_passes = false;
                    break;
                }
            }
            if !has_passes {
                self.queue.submit(std::iter::once(encoder.finish()));
                output.present();
                continue;
            }

            // Camera has passes, draw them
            for layer in &camera_lock.render_layers {
                let layer = render_layers::get_layer_by_name(layer.to_string());
                let layer = match layer {
                    Some(l) => l,
                    None => continue,
                };
                let layer_lock = layer.read();

                // Do a pass
                for (_pass_id, pass_data) in &layer_lock.passes {
                    // Prepare data
                    let pass_lock = pass_data.write();
                    let material_lock = pass_lock.material.read();
                    let pipeline = Arc::clone(&material_lock.get_pipeline(self));
                    let texture_bind_group =
                        Arc::clone(&material_lock.get_texture_bind_group(self));

                    // Create the pass
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_texture.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });
                    render_pass.set_pipeline(&pipeline);
                    render_pass.set_bind_group(0, &texture_bind_group, &[]);
                    render_pass.set_bind_group(1, &camera_lock.bind_group, &[]);
                    render_pass.set_vertex_buffer(0, pass_lock.buffer.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        pass_lock.buffer.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    // println!("Drawing mesh");
                    // println!("{} vertices", pass_lock.buffer.vertex_count);
                    // println!("{} indices", pass_lock.buffer.index_count);
                    render_pass.draw_indexed(0..pass_lock.buffer.index_count, 0, 0..1);
                    drop(render_pass); // Required to release the borrow of encoder
                }
            }

            // submit will accept anything that implements IntoIter
            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        Ok(())
    }
}
