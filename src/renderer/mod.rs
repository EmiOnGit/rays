pub mod render_pipeline;
pub mod image_util;
mod render;

use glam::Vec3;
use image::{RgbaImage, Rgba, Pixel};
use log::warn;
use wgpu::{Device, Texture};
use winit::dpi::PhysicalSize;

use crate::{
    math::ray::Ray,
    scene::Scene,
};

use self::image_util::ImageSize;

pub struct Renderer {
    /// This buffer can be used to draw on
    pub image_buffer: RgbaImage,
}

impl Renderer {
    const BACKGROUND_COLOR: Rgba<u8> = Rgba([0, 0, 0, 255]);

    pub fn new(size: PhysicalSize<u32>) -> Self {
        // output texture
        let image_buffer = RgbaImage::from_pixel(size.width, size.height, Rgba([0, 0, 0, 0]));
        Self { image_buffer }
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
        self.image_buffer =
            RgbaImage::from_pixel(new_size.width, new_size.height, Rgba([0, 0, 0, 0]));
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
fn trace_ray(ray: Ray, scene: &Scene) -> Option<Rgba<f32>> {
    let mut hit_distance = f32::MAX;
    let mut closest_sphere = None;

    for sphere in &scene.spheres {
        let ray_d = Ray::new(ray.origin - sphere.center, ray.direction);
        let a = ray_d.direction.dot(ray_d.direction);
        let b = 2. * ray_d.origin.dot(ray_d.direction);
        let c = ray_d.origin.dot(ray_d.origin) - sphere.radius * sphere.radius;
        let discriminant = b * b - 4. * a * c;
        if discriminant < 0. {
            continue;
        }
        // let t0 = (-b + discriminant.sqrt()) / (2. * a);
        let closest_t = (-b - discriminant.sqrt()) / (2. * a);
        if closest_t > 0. && closest_t < hit_distance {
            hit_distance = closest_t;
            closest_sphere = Some(sphere);
        }
    }
    let Some(sphere) = closest_sphere else {
        return None;
    };
    let ray_d = Ray::new(ray.origin - sphere.center, ray.direction);

    let light_dir = Vec3::new(1., 1., 1.).normalize();

    // let h0 = ray.at(t0);
    let hit_point = ray_d.at(hit_distance);
    let normal = hit_point.normalize();
    let d = normal.dot(-light_dir).max(0.);
    let mut color = sphere.albedo;
    color.apply_without_alpha(|c| c * d);
    Some(color)
}
