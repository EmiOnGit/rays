#![allow(dead_code)]
pub mod compute_pipeline;
pub mod image_util;
mod render;

pub mod render_pipeline;

use image::{Rgba, Rgba32FImage};
use log::warn;
use wgpu::{Device, Texture};
use winit::dpi::PhysicalSize;

use self::image_util::ImageSize;
pub struct Renderer {
    /// This buffer can be used to draw on
    pub image_buffer: Rgba32FImage,
    acc_buffer: Rgba32FImage,
    acc_frame: usize,
}

impl Renderer {
    pub fn new(size: PhysicalSize<u32>) -> Self {
        // output texture
        let image_buffer =
            Rgba32FImage::from_pixel(size.width, size.height, Rgba([0., 0., 0., 0.]));
        let acc_buffer = Rgba32FImage::from_pixel(size.width, size.height, Rgba([0., 0., 0., 0.]));
        Self {
            image_buffer,
            acc_buffer,
            acc_frame: 1,
        }
    }
    pub fn size(&self) -> ImageSize {
        let dimensions = self.image_buffer.dimensions();
        ImageSize::new(dimensions.0, dimensions.1)
    }
    pub fn get_image(&self) -> &Rgba32FImage {
        &self.image_buffer
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        warn!(
            "image buffer now has size: {}",
            self.image_buffer.pixels().len()
        );
        self.acc_frame = 0;
        self.acc_buffer =
            Rgba32FImage::from_pixel(new_size.width, new_size.height, Rgba([0., 0., 0., 0.]));
        self.image_buffer =
            Rgba32FImage::from_pixel(new_size.width, new_size.height, Rgba([0., 0., 0., 0.]));
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
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        })
    }
}
