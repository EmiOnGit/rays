use std::{
    any::type_name,
    iter::{self},
};

use bytemuck::Pod;
use egui_wgpu::renderer::ScreenDescriptor;
use log::info;
use wgpu::{
    util::DeviceExt, Buffer, BufferAddress, CommandEncoder, Device, Features, Label, Limits,
    PresentMode,
};
use winit::{
    dpi::PhysicalSize,
    event::{KeyboardInput, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

use crate::{
    camera::CameraUniform,
    globals::Globals,
    renderer::{compute_pipeline::ComputePipeline, render_pipeline::RenderPipeline, Renderer},
    scene::Scene,
    timer::Timer,
    ui::UiManager,
};

pub struct App {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,

    queue: wgpu::Queue,
    pub device: wgpu::Device,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: winit::window::Window,

    renderer: Renderer,
    pub scene: Scene,
    globals: Globals,
    camera_uniform: CameraUniform,
    render_pipeline: RenderPipeline,
    compute_pipeline: ComputePipeline,
    timer: Timer,
    ui_manager: UiManager,
    scale_factor: f32,
}
impl App {
    pub async fn new(window: Window, event_loop: &EventLoop<()>) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let wgpu_instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { wgpu_instance.create_surface(&window) }.unwrap();

        // Adapter to gpu
        // Can retrieve information about the graphics card directly
        let adapter = wgpu_instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        info!("Gpu used: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits: Limits::default(),
                    label: Label::Some("gpu device. Used to open connections to the gpu"),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Mailbox,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);
        let scene = Scene::example_scene();
        let renderer = Renderer::new(size);

        let input_texture = renderer.create_input_texture(&device);
        let input_texture_view = input_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let render_pipeline =
            RenderPipeline::new(&device, &surface_config, input_texture_view, input_texture);
        let compute_pipeline = ComputePipeline::new(&device, &scene);
        let globals = Globals::default();
        let timer = Timer::new();
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [surface_config.width, surface_config.height],
            pixels_per_point: 1.,
        };
        let camera_uniform = CameraUniform::from(&scene.camera);
        let ui_manager = UiManager::new(&device, surface_format, screen_descriptor, event_loop);
        Self {
            window,
            surface,
            device,
            render_pipeline,
            compute_pipeline,
            camera_uniform,
            surface_config,
            timer,
            globals,
            queue,
            renderer,
            ui_manager,
            scale_factor: 0.,
            scene,
        }
    }
    pub fn clear_renderer(&mut self) {
        self.renderer.reset_acc();
        let input_texture = self.renderer.create_input_texture(&self.device);
        self.render_pipeline.set_input_texture(input_texture);
    }
    pub fn resize(&mut self, new_size: PhysicalSize<u32>, scale_factor: Option<f32>) {
        self.renderer.resize(new_size);
        self.scene.camera.resize(new_size);
        // resize surface
        self.surface_config.height = new_size.height;
        self.surface_config.width = new_size.width;
        self.surface.configure(&self.device, &self.surface_config);
        let input_texture = self.renderer.create_input_texture(&self.device);
        self.render_pipeline.set_input_texture(input_texture);
        if let Some(scale_factor) = scale_factor {
            self.scale_factor = scale_factor;
        }
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.surface_config.width, self.surface_config.height],
            pixels_per_point: self.scale_factor,
        };
        self.ui_manager.resize(screen_descriptor);
    }
    pub fn handle_window_event(&mut self, window_event: &WindowEvent) {
        self.ui_manager.handle_window_event(window_event);
    }
    pub fn render_ui(&mut self) {
        let renderer_need_reset = self.ui_manager.run(
            &self.device,
            &self.queue,
            &self.window,
            &mut self.scene,
            &mut self.globals,
        );
        if renderer_need_reset {
            self.clear_renderer();
        }
    }
    pub fn prepare(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        self.render_pipeline.surface_texture = Some(surface_texture);
        self.render_pipeline.prepare_bind_group(&self.device);
        Ok(())
    }
    pub fn queue(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // write data to gpu

        self.ui_manager
            .update_buffers(&mut encoder, &self.device, &self.queue);

        {
            let mut encoder = encoder
                // compute_pipeline
                .write_buffer(
                    self.globals,
                    &self.compute_pipeline.globals_buffer,
                    &self.device,
                )
                .write_buffer(
                    self.camera_uniform,
                    &self.compute_pipeline.camera_buffer,
                    &self.device,
                )
                .write_slice_buffer(
                    &self.scene.spheres,
                    &self.compute_pipeline.sphere_buffer,
                    &self.device,
                )
                .write_slice_buffer(
                    &self.scene.materials,
                    &self.compute_pipeline.material_buffer,
                    &self.device,
                )
                // render pipeline
                .write_buffer(
                    self.renderer.acc_frame,
                    &self.render_pipeline.acc_frame_buffer,
                    &self.device,
                );

            let compute_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Compute bind group"),
                layout: &self.compute_pipeline.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            self.compute_pipeline
                                .globals_buffer
                                .as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(
                            self.compute_pipeline
                                .camera_buffer
                                .as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(
                            &self.render_pipeline.input_texture_view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Buffer(
                            self.compute_pipeline
                                .sphere_buffer
                                .as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Buffer(
                            self.compute_pipeline
                                .material_buffer
                                .as_entire_buffer_binding(),
                        ),
                    },
                ],
            });

            {
                let size = self.renderer.image_buffer.dimensions();
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: "Compute Pass".into(),
                });
                compute_pass.set_pipeline(&self.compute_pipeline.pipeline);
                compute_pass.set_bind_group(0, &compute_bind_group, &[]);
                // defined in the shader
                const WORKGROUP_SIZE: u32 = 16;
                compute_pass.dispatch_workgroups(
                    size.0 / WORKGROUP_SIZE,
                    size.1 / WORKGROUP_SIZE,
                    1,
                );
            }

            let render_bind_group = self.render_pipeline.bind_group.as_ref().unwrap();
            {
                let view = self.render_pipeline.surface_texture_view();
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.render_pipeline.pipeline);
                render_pass.set_bind_group(0, render_bind_group, &[]);
                render_pass.draw(0..6, 0..1);
            }
            // write to egui
            {
                let view = self.render_pipeline.surface_texture_view();

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Gui Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                self.ui_manager.render(&mut render_pass);
            }
            // submit will accept anything that implements IntoIter
            self.queue.submit(iter::once(encoder.finish()));
            self.render_pipeline
                .surface_texture
                .take()
                .unwrap()
                .present();
        }
    }

    pub fn update(&mut self) {
        self.timer.update();
        self.camera_uniform = CameraUniform::from(&self.scene.camera);
        self.globals.seed = fastrand::u32(..);
        self.renderer.acc_frame += 1;
    }
    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn handle_keyboard_input(&mut self, input: &KeyboardInput) {
        let moved = self.scene.camera.on_keyboard_event(input, self.timer.dt());
        if moved {
            self.clear_renderer();
        }
    }
}

trait BufferSet {
    fn write_buffer<T: Pod>(self, obj: T, destination: &Buffer, device: &Device) -> Self;
    fn write_slice_buffer<T: Pod>(self, obj: &[T], destination: &Buffer, device: &Device) -> Self;
}
impl BufferSet for CommandEncoder {
    fn write_buffer<T: Pod>(mut self, obj: T, destination: &Buffer, device: &Device) -> Self {
        let t_size = std::mem::size_of::<T>();
        let name = type_name::<T>();
        let source = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: format!("{name} buffer").as_str().into(),
            contents: bytemuck::cast_slice(&[obj]),
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        self.copy_buffer_to_buffer(&source, 0, &destination, 0, t_size as BufferAddress);
        self
    }
    fn write_slice_buffer<T: Pod>(
        mut self,
        obj: &[T],
        destination: &Buffer,
        device: &Device,
    ) -> Self {
        let t_size = std::mem::size_of::<T>() * obj.len();
        let name = type_name::<T>();
        let source = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: format!("{name} buffer").as_str().into(),
            contents: bytemuck::cast_slice(obj),
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        self.copy_buffer_to_buffer(&source, 0, &destination, 0, t_size as BufferAddress);
        self
    }
}
