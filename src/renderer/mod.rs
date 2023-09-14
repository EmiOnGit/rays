pub mod material;
pub mod render_pipeline;
pub mod sphere;

use glam::Vec3;
use image::{Pixel, Rgba, RgbaImage};
use log::warn;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use winit::dpi::PhysicalSize;

use crate::{
    camera::Camera,
    math::{color_f32_to_u8, ray::Ray},
    scene::Scene,
};

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

    pub fn get_image(&self) -> &RgbaImage {
        &self.image_buffer
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        warn!(
            "image buffer now has size: {}",
            self.image_buffer.pixels().len()
        );
        self.image_buffer =
            RgbaImage::from_pixel(new_size.width, new_size.height, Rgba([0, 0, 0, 0]));
    }

    pub fn render(&mut self, camera: &Camera, scene: &Scene) {
        let mut colors = Vec::with_capacity(camera.ray_directions.len());
        camera
            .ray_directions
            .par_iter()
            .map(|ray_direction| {
                let ray = Ray::new(camera.position, *ray_direction);

                match trace_ray(ray, scene) {
                    Some(color) => color_f32_to_u8(color),
                    None => Self::BACKGROUND_COLOR,
                }
            })
            .collect_into_vec(&mut colors);

        for (i, pixel) in self.image_buffer.pixels_mut().enumerate() {
            *pixel = colors[i];
        }
    }
}
fn trace_ray(ray: Ray, scene: &Scene) -> Option<Rgba<f32>> {
    let mut hit_distance = f32::MAX;
    let mut closest_sphere = None;

    for sphere in &scene.spheres[..] {
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
