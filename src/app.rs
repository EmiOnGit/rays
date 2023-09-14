use image::{Rgba, RgbaImage};
use log::info;
use wgpu::{Device, Features, Label, Limits, PresentMode, SurfaceConfiguration, Texture};
use winit::{
    dpi::PhysicalSize,
    event::{KeyboardInput, WindowEvent},
    window::Window,
};

use crate::{
    camera::Camera,
    renderer::{render_pipeline::RenderPipeline, Renderer},
    scene::Scene,
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
    render_pipeline: RenderPipeline,
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
        // output texture
        let image_buffer = RgbaImage::from_pixel(size.width, size.width, Rgba([0, 0, 0, 0]));
        let output_texture = create_input_texture(&image_buffer, &device);
        let output_texture_view =
            output_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let render_pipeline = RenderPipeline::new(
            &device,
            &surface_config,
            output_texture_view,
            output_texture,
        );

        let camera = Camera::new(45., 0.1, 100., size.width as f32, size.height as f32);
        let scene = Scene::example_scene();
        let renderer = Renderer::new(size);
        Self {
            window,
            surface,
            device,
            render_pipeline,
            surface_config,
            queue,
            camera,
            renderer,
            scene,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        println!("resizing with new size: {:?}", new_size);
        self.renderer.resize(new_size);
        self.camera.resize(new_size);
        // resize surface
        self.surface_config.height = new_size.height;
        self.surface_config.width = new_size.width;
        self.surface.configure(&self.device, &self.surface_config);
    }
    pub fn queue(&mut self) {
        let image = self.renderer.get_image();
        let dimensions = image.dimensions();

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.render_pipeline.input_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            self.render_pipeline.input_texture.size(),
        );
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
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
        self.queue.submit(Some(encoder.finish()));
        self.render_pipeline
            .surface_texture
            .take()
            .unwrap()
            .present();
    }

    pub fn prepare(&mut self) -> Result<(), wgpu::SurfaceError> {
        let image = self.renderer.get_image();
        let input_texture = create_input_texture(image, &self.device);

        let surface_texture = self.surface.get_current_texture()?;
        self.render_pipeline.set_input_texture(input_texture);
        self.render_pipeline.set_surface_texture(surface_texture);
        self.render_pipeline.prepare_bind_group(&self.device);
        Ok(())
    }
    pub fn update(&mut self) {
        self.renderer.render(&self.camera, &self.scene);
    }
    pub fn window(&self) -> &Window {
        &self.window
    }
    pub fn input(&self, _event: &WindowEvent) -> bool {
        false
    }
    pub fn handle_keyboard_input(&mut self, input: &KeyboardInput) {
        self.camera.on_keyboard_event(input, 1.);
    }
}

fn create_input_texture(image_buffer: &RgbaImage, device: &Device) -> Texture {
    let texture_size = wgpu::Extent3d {
        width: image_buffer.width(),
        height: image_buffer.height(),
        depth_or_array_layers: 1,
    };
    let output_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Output texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    output_texture
}
