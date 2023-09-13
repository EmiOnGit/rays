mod render_pipeline;

use glam::Vec3;
use image::{Rgba, RgbaImage};
use log::info;
use rayon::prelude::{IntoParallelIterator, ParallelIterator, IndexedParallelIterator};
use wgpu::{Device, Label, PresentMode, Texture};
use winit::{event::WindowEvent, window::Window};

use crate::{
    camera::Camera,
    math::{color_f32_to_u8, ray::Ray},
};

use self::render_pipeline::RenderPipeline;

pub struct State {
    /// Surface of the window we draw on
    surface: wgpu::Surface,
    background_color: Rgba<u8>,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: RenderPipeline,
    /// This buffer can be used to draw on
    pub image_buffer: RgbaImage,
    pub pixel_data: Vec<Rgba<u8>>,
    output_texture_view: wgpu::TextureView,
    output_texture: wgpu::Texture,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Window,
}

impl State {
    // Creating some of the wgpu types requires async code
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
        info!("Adapter used: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits: wgpu::Limits::default(),
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
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::default(),
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let render_pipeline = RenderPipeline::new(&device, &config);

        // output texture
        let image_buffer = RgbaImage::from_pixel(size.width, size.width, Rgba([0, 0, 0, 0]));
        let output_texture = create_texture(&image_buffer, &device);
        let output_texture_view =
            output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            window,
            surface,
            render_pipeline,
            device,
            output_texture,
            output_texture_view,
            background_color: [0, 0, 0, 255].into(),
            queue,
            config,
            size,
            image_buffer,
            pixel_data: Vec::new(),
        }
    }
    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.image_buffer = RgbaImage::from_pixel(new_size.width, new_size.height, Rgba([0, 0, 0, 0]));
            self.output_texture = create_texture(&self.image_buffer, &self.device);
            self.output_texture_view =
            self.output_texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let dimensions = self.image_buffer.dimensions();

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.image_buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            self.output_texture.size(),
        );
        let current_texture = self.surface.get_current_texture()?;
        let view = current_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let render_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render bind group"),
            layout: &self.render_pipeline.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.output_texture_view),
            }],
        });
        {
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
            render_pass.set_bind_group(0, &render_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(Some(encoder.finish()));
        current_texture.present();

        Ok(())
    }
    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
    pub fn update(&mut self, camera: &Camera) {
        let height = self.image_buffer.height();
        let width = self.image_buffer.width();

        (0..height * width).into_par_iter().map(|i| {
            let Some(ray_direction) = camera.ray_directions.get(i as usize) else {
                return self.background_color;
            };

            let ray = Ray::new(camera.position, *ray_direction);

            match trace_ray(&ray) {
                Some(color) => color_f32_to_u8(color),
                None => self.background_color,
            }
        })
        .collect_into_vec(&mut self.pixel_data);

        for (i, pixel) in self.image_buffer.pixels_mut().enumerate() {
            *pixel = self.pixel_data[i];
        }
       

    }
}
fn trace_ray(ray: &Ray) -> Option<Rgba<f32>> {
    let radius = 0.5;
    let a = ray.direction.dot(ray.direction);
    let b = 2. * ray.origin.dot(ray.direction);
    let c = ray.origin.dot(ray.origin) - radius * radius;
    let discriminant = b * b - 4. * a * c;
    if discriminant < 0. {
        return None;
    }

    let light_dir = Vec3::new(-1., -1., -1.).normalize();

    // let t0 = (-b + discriminant.sqrt()) / (2. * a);
    let closest_t = (-b - discriminant.sqrt()) / (2. * a);
    // let h0 = ray.at(t0);
    let hit_point = ray.at(closest_t);
    let normal = hit_point.normalize();
    let d = normal.dot(-light_dir).max(0.);
    let mut color = Vec3::new(1., 1., 1.);
    color *= d;
    Some(Rgba([color.x, color.y, color.z, 1.]))
}

fn create_texture(image_buffer: &RgbaImage, device: &Device) -> Texture {
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
