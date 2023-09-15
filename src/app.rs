use std::iter::{self};

use egui_wgpu::renderer::ScreenDescriptor;
use glam::{Mat4, Vec2};
use log::info;
use wgpu::{util::DeviceExt, BufferAddress, Features, Label, Limits, PresentMode};
use winit::{
    dpi::PhysicalSize,
    event::{KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

use crate::{
    camera::Camera,
    globals::Globals,
    material::Material,
    renderer::{compute_pipeline::ComputePipeline, render_pipeline::RenderPipeline, Renderer},
    scene::Scene,
    sphere::Sphere,
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
    pub camera: Camera,
    scene: Scene,
    globals: Globals,

    render_pipeline: RenderPipeline,
    compute_pipeline: ComputePipeline,
    timer: Timer,
    ui_manager: UiManager,
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
        let camera = Camera::new(45., 0.1, 100., size.width as f32, size.height as f32);
        let scene = Scene::example_scene();
        let renderer = Renderer::new(size);

        let input_texture = renderer.create_input_texture(&device);
        let input_texture_view = input_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let render_pipeline =
            RenderPipeline::new(&device, &surface_config, input_texture_view, input_texture);
        let compute_pipeline = ComputePipeline::new(&device);
        let globals = Globals {
            seed: 105,
            viewport: Vec2::ONE * 1000.,
            inverse_projection: Mat4::IDENTITY,
            inverse_view: Mat4::IDENTITY,
        };
        let timer = Timer::new();
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [surface_config.width, surface_config.height],
            pixels_per_point: 1.,
        };
        let ui_manager = UiManager::new(&device, surface_format, screen_descriptor, event_loop);
        Self {
            window,
            surface,
            device,
            render_pipeline,
            compute_pipeline,
            surface_config,
            timer,
            globals,
            queue,
            camera,
            renderer,
            ui_manager,

            scene,
        }
    }
    pub fn clear_renderer(&mut self) {
        self.renderer.reset_acc();
    }
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.renderer.resize(new_size);
        self.camera.resize(new_size);
        // resize surface
        self.surface_config.height = new_size.height;
        self.surface_config.width = new_size.width;
        self.surface.configure(&self.device, &self.surface_config);
        let input_texture = self.renderer.create_input_texture(&self.device);
        self.render_pipeline.set_input_texture(input_texture);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.surface_config.width, self.surface_config.height],
            pixels_per_point: 1.,
        };
        self.ui_manager.resize(screen_descriptor);
    }
    pub fn handle_window_event(&mut self, window_event: &WindowEvent) {
        self.ui_manager.handle_window_event(window_event);
    }
    pub fn render_ui(&mut self) {
        self.ui_manager.run(&self.device, &self.queue, &self.window);
    }
    pub fn prepare(&mut self) -> Result<(), wgpu::SurfaceError> {
        let input_texture = self.renderer.create_input_texture(&self.device);
        self.render_pipeline.set_input_texture(input_texture);

        let surface_texture = self.surface.get_current_texture()?;
        self.render_pipeline.surface_texture = Some(surface_texture);

        self.render_pipeline.prepare_bind_group(&self.device);
        Ok(())
    }
    pub fn queue(&mut self) {
        let size = self.renderer.size();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // write data to gpu

        self.ui_manager
            .update_buffers(&mut encoder, &self.device, &self.queue);

        {
            let globals_size = std::mem::size_of::<Globals>();
            let globals_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: "Globals buffer".into(),
                        contents: bytemuck::cast_slice(&[self.globals]),
                        usage: wgpu::BufferUsages::COPY_SRC,
                    });

            encoder.copy_buffer_to_buffer(
                &globals_buffer,
                0,
                &self.compute_pipeline.globals_buffer,
                0,
                globals_size as BufferAddress,
            );
            let sphere = self.scene.spheres[0];
            let sphere_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: "Sphere buffer".into(),
                    contents: bytemuck::cast_slice(&[sphere]),
                    usage: wgpu::BufferUsages::COPY_SRC,
                });
            let sphere_size = std::mem::size_of::<Sphere>();
            encoder.copy_buffer_to_buffer(
                &sphere_buffer,
                0,
                &self.compute_pipeline.sphere_buffer,
                0,
                sphere_size as BufferAddress,
            );
            let material_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: "Material buffer".into(),
                        contents: bytemuck::cast_slice(&[self.scene.materials[0]]),
                        usage: wgpu::BufferUsages::COPY_SRC,
                    });
            let material_size = std::mem::size_of::<Material>();
            encoder.copy_buffer_to_buffer(
                &material_buffer,
                0,
                &self.compute_pipeline.material_buffer,
                0,
                material_size as BufferAddress,
            );
        }
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
                    resource: wgpu::BindingResource::TextureView(
                        &self.render_pipeline.input_texture_view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(
                        self.compute_pipeline
                            .sphere_buffer
                            .as_entire_buffer_binding(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(
                        self.compute_pipeline
                            .material_buffer
                            .as_entire_buffer_binding(),
                    ),
                },
            ],
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: "Compute Pass".into(),
            });
            compute_pass.set_pipeline(&self.compute_pipeline.pipeline);
            compute_pass.set_bind_group(0, &compute_bind_group, &[]);
            // defined in the shader
            const WORKGROUP_SIZE: u32 = 8;
            compute_pass.dispatch_workgroups(
                size.width / WORKGROUP_SIZE,
                size.height / WORKGROUP_SIZE,
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

    pub fn update(&mut self) {
        self.timer.update();
        self.globals.inverse_projection = self.camera.inverse_projection;
        self.globals.inverse_view = self.camera.inverse_view;
        self.globals.viewport = Vec2::new(self.camera.viewport_width, self.camera.viewport_height);
        // self.renderer.render(&self.camera, &self.scene);
        // self.renderer.update_image_buffer();
    }
    pub fn window(&self) -> &Window {
        &self.window
    }
    pub fn input(&self, _event: &WindowEvent) -> bool {
        false
    }
    pub fn handle_keyboard_input(&mut self, input: &KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            if code == VirtualKeyCode::P {
                self.globals.seed = self.globals.seed.wrapping_add(5);
            }

            if code == VirtualKeyCode::O {
                self.globals.seed = self.globals.seed.wrapping_sub(5);
            }
            if code == VirtualKeyCode::L {
                self.scene.spheres[0].radius += 0.1;
            }
        }
        let moved = self.camera.on_keyboard_event(input, self.timer.dt());
        if moved {
            self.clear_renderer();
        }
    }
}
