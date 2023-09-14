use std::iter::{self};

use log::info;
use wgpu::{Features, Label, Limits, PresentMode, SurfaceConfiguration, util::DeviceExt};
use winit::{
    dpi::PhysicalSize,
    event::{KeyboardInput, WindowEvent, VirtualKeyCode},
    window::Window,
};

use crate::{
    camera::Camera,
    renderer::{render_pipeline::RenderPipeline, Renderer, compute_pipeline::ComputePipeline},
    scene::Scene,
    timer::Timer, globals::Globals,
};

pub struct App {
    surface: wgpu::Surface,
    surface_config: SurfaceConfiguration,

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
}
impl App {
    pub async fn new(window: Window) -> Self {
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
        let compute_pipeline =
            ComputePipeline::new(&device);
        let globals = Globals {
            seed: 105,
        };
        let timer = Timer::new();
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
    }

    pub fn prepare(&mut self) -> Result<(), wgpu::SurfaceError> {
        let input_texture = self.renderer.create_input_texture(&self.device);
        self.render_pipeline.set_input_texture(input_texture);

        let surface_texture = self.surface.get_current_texture()?;
        self.render_pipeline.surface_texture = Some(surface_texture);

        self.render_pipeline.prepare_bind_group(&self.device);
        // self.compute_pipeline.prepare_bind_group(&self.device);
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
        {
            let globals_size = std::mem::size_of::<Globals>();
           
            let globals_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
                label: "Globals buffer".into(),
                contents: bytemuck::cast_slice(&[self.globals]), 
                usage: wgpu::BufferUsages::COPY_SRC ,

            });
            encoder.copy_buffer_to_buffer(
                &globals_buffer,
                0,
                &self.compute_pipeline.globals_buffer,
                0,
                globals_size as u64,
            );  
        }
        let compute_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute bind group"),
            layout: &self.compute_pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(self.compute_pipeline.globals_buffer.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.render_pipeline.input_texture_view),
                },
            ],
        });
      
        // self.queue.write_texture(
        //     wgpu::ImageCopyTexture {
        //         texture: &self.render_pipeline.input_texture,
        //         mip_level: 0,
        //         origin: wgpu::Origin3d::ZERO,
        //         aspect: wgpu::TextureAspect::All,
        //     },
        //     self.renderer.get_image().as_bytes(),
        //     wgpu::ImageDataLayout {
        //         offset: 0,
        //         bytes_per_row: Some(4 * 4 * size.width),
        //         rows_per_image: Some(size.height),
        //     },
        //     size.into(),
        // );
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: "Compute Pass".into() });
            compute_pass.set_pipeline(&self.compute_pipeline.pipeline);
            compute_pass.set_bind_group(0, &compute_bind_group, &[]);
            // defined in the shader
            const WORKGROUP_SIZE: u32 = 8;
            compute_pass.dispatch_workgroups(size.width / WORKGROUP_SIZE,size.height / WORKGROUP_SIZE ,1);
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline.pipeline);
            render_pass.set_bind_group(0, render_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
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
        self.renderer.render(&self.camera, &self.scene);
        self.renderer.update_image_buffer();
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
                println!("sjklhd");
            }

            if code == VirtualKeyCode::O {
                self.globals.seed = self.globals.seed.wrapping_sub(5);
            }
        }
        let moved = self.camera.on_keyboard_event(input, self.timer.dt());
        if moved {
            self.clear_renderer();
        }
    }
}
