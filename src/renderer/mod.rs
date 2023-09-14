pub mod image_util;
mod render;
pub mod render_pipeline;

use image::{Rgba, RgbaImage, Rgb32FImage, Rgb};
use log::warn;
use wgpu::{Device, Texture};
use winit::dpi::PhysicalSize;

use self::image_util::ImageSize;

pub struct Renderer {
    /// This buffer can be used to draw on
    pub image_buffer: RgbaImage,
    acc_buffer: Rgb32FImage,
    acc_frame: usize,
    seed: u32,
}

impl Renderer {
    pub fn new(size: PhysicalSize<u32>) -> Self {
        // output texture
        let image_buffer = RgbaImage::from_pixel(size.width, size.height, Rgba([0, 0, 0, 0]));
        let acc_buffer = Rgb32FImage::from_pixel(size.width, size.height, Rgb([0., 0., 0.]));
        Self {
            image_buffer,
            acc_buffer,
            acc_frame: 1,
            seed: 0,
        }
    }
    pub fn size(&self) -> ImageSize {
        let dimensions = self.image_buffer.dimensions();
        ImageSize::new(dimensions.0, dimensions.1)
    }
    pub fn get_image(&self) -> &RgbaImage {
        &self.image_buffer
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        warn!(
            "image buffer now has size: {}",
            self.image_buffer.pixels().len()
        );
        self.acc_frame = 0;
        self.acc_buffer =
            Rgb32FImage::from_pixel(new_size.width, new_size.height, Rgb([0., 0., 0.]));
        self.image_buffer =
            RgbaImage::from_pixel(new_size.width, new_size.height, Rgba([0, 0, 0, 0]));
    }
    pub fn reset_acc(&mut self) {
        self.acc_frame = 0;

        self.acc_buffer.fill(0.);
    }

    pub fn create_input_texture(&self, device: &Device) -> Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Input texture"),
            size: self.size().into(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }
}
