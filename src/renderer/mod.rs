mod render_pipeline;

use glam::Vec3;
use image::{Rgba, RgbaImage};
use log::info;
use wgpu::{Label, PresentMode, Device, Texture, Queue};
use winit::{event::WindowEvent, window::Window};

use crate::math::{ray::Ray, color_f32_to_u8};

use self::render_pipeline::RenderPipeline;

pub struct State {
    /// Surface of the window we draw on
    surface: wgpu::Surface,
    background_color: wgpu::Color,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: RenderPipeline,
    /// This buffer can be used to draw on
    image_buffer: RgbaImage,
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
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let render_pipeline = RenderPipeline::new(&device, &config);

        // output texture
        let image_buffer = RgbaImage::from_pixel(size.width, size.width, Rgba([0,0,0,0]));
        let output_texture = write_texture(&image_buffer, &device, &queue);
        let output_texture_view =
            output_texture.create_view(&wgpu::TextureViewDescriptor::default());
       
        Self {
            window,
            surface,
            render_pipeline,
            device,
            output_texture,
            output_texture_view,
            background_color: wgpu::Color::BLUE,
            queue,
            config,
            size,
            image_buffer,
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
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let dimensions = self.image_buffer.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
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
            texture_size,
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
                        load: wgpu::LoadOp::Clear(self.background_color),
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
    pub fn update(&mut self) {
        let dimensions = self.image_buffer.dimensions();
        for (x,y,p) in self.image_buffer.enumerate_pixels_mut() {
            let x = x as f32 / dimensions.0 as f32;
            let y = y as f32 / dimensions.1 as f32;
            *p = color_f32_to_u8(update_pixel(x,y));
        }
    }


}
fn update_pixel(x: f32, y: f32) -> Rgba<f32> {
    let x = x * 2. - 1.; // map to -1 .. 1
    let y = y * 2. - 1.; // map to -1 .. 1

    let origin =  Vec3::Z;
    let direction = Vec3::new(x, y, -1.);
    let ray = Ray::new(origin, direction);
    let radius = 0.5;
    let light_dir = Vec3::new(-1.,-1.,-1.).normalize();

    let a = direction.dot(direction);
    let b = 2. * origin.dot(direction);
    let c = origin.dot(origin) - radius * radius;
    let discriminant = b * b - 4. * a * c;

    if discriminant < 0. {
        return Rgba([0.7,0.6,0.6,1.])
    }
    // let t0 = (-b + discriminant.sqrt()) / (2. * a);
    let closest_t = (-b - discriminant.sqrt()) / (2. * a);
    // let h0 = ray.at(t0);
    let hit_point = ray.at(closest_t);
    let normal = hit_point.normalize();
    let d = normal.dot(-light_dir).max(0.);
    let mut color = Vec3::new(1.,0.,1.);
    color *= d;
    Rgba([color.x, color.y, color.z, 1.])
}

fn write_texture(image_buffer: &RgbaImage, device: &Device, queue: &Queue) -> Texture {
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
   
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &output_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &image_buffer,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * image_buffer.width()),
            rows_per_image: Some(image_buffer.height()),
        },
        texture_size,
    );
    output_texture
}