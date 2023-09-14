use glam::Vec3;
use image::{DynamicImage, Pixel, Rgb};
#[cfg(feature = "rayon")]
use rayon::prelude::ParallelIterator;
use crate::{
    camera::Camera,
    math::{self, ray::Ray},
    scene::Scene,
};

use super::{Renderer, image_util};

impl Renderer {
    pub fn render(&mut self, camera: &Camera, scene: &Scene) {
        self.acc_frame = self.acc_frame + 1;
        let colors: Vec<Rgb<f32>> = self.size()
            .into_iter()
            .map(|(y, x)| self.per_pixel(x, y, camera, scene))
            .collect();
        
        for (i, pixel) in self.acc_buffer.pixels_mut().enumerate() {
            let c = colors[i];
            *pixel = [pixel.0[0] + c[0], pixel.0[1] + c[1], pixel.0[2] + c[2]].into();
        }
        let mut i = self.acc_buffer.clone();
        image_util::iter_mut_image_buffer(&mut i).for_each(|pixel| {
            assert_ne!(self.acc_frame, 0);
            *pixel = *pixel / self.acc_frame as f32;
        });
        
        self.image_buffer = DynamicImage::from(i).into_rgba8();
        self.seed = math::pcg_hash(self.seed);
    }
    pub fn per_pixel(&self, x: usize, y: usize, camera: &Camera, scene: &Scene) -> Rgb<f32> {
        let direction = camera.ray_directions[x + y * self.size().width as usize];
        let mut ray = Ray::new(camera.position, direction);
        let bounces = 3;
        let mut color_acc = Vec3::ZERO;
        let mut mult = 1.;
        let mut bounced = bounces;
        for i in 0..bounces {
            let payload = self.trace_ray(ray.clone(), scene);
            let Some(payload) = payload else {
                color_acc += Vec3::new(0.8, 0.8, 1.);
                bounced = i + 1;
                break;
            };
            let sphere = &scene.spheres[payload.index];

            let mat = scene.material(sphere);
            ray.origin = payload.world_position - payload.world_normal * 0.0001;
            let r1 = math::rand(payload.hit_distance.to_bits() + self.seed) - 0.5;
            let r2 = math::rand((r1 * f32::MAX).to_bits()) - 0.5;
            let r3 = math::rand((r2 * f32::MAX).to_bits()) - 0.5;
            let n = payload.world_normal + mat.roughness * Vec3::new(r1, r2, r3);
            
            ray.direction = ray.direction - 2. * (ray.direction.dot(n) * n);
            let light_dir = Vec3::new(1., 1., 1.).normalize();
            let light_intensity = payload.world_normal.normalize().dot(-light_dir).max(0.);
            let mut color = scene.material(sphere).albedo;
            color.apply(|c| c * light_intensity * mult);
            color_acc += Vec3::from(color.0);
            mult *= 0.7;
        }
        let raw_c = (color_acc / bounced as f32).to_array();
        let color = Rgb(raw_c);
        color
    }
    pub fn trace_ray(&self, ray: Ray, scene: &Scene) -> Option<HitPayload> {
        let mut hit_distance = f32::MAX;
        let mut closest_index = -1;
        for i in 0..scene.spheres.len() {
            let sphere = &scene.spheres[i];
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
                closest_index = i as i32;
            }
        }
        if closest_index < 0 {
            self.miss()
        } else {
            Some(self.closest_hit(ray, hit_distance, closest_index as usize, scene))
        }
    }
    pub fn closest_hit(
        &self,
        mut ray: Ray,
        hit_distance: f32,
        object_index: usize,
        scene: &Scene,
    ) -> HitPayload {
        let sphere = &scene.spheres[object_index];
        let origin = ray.origin - sphere.center;
        ray.origin = origin;
        // let h0 = ray.at(t0);
        let hit_point = ray.at(hit_distance);
        let world_normal = hit_point.normalize();
        HitPayload {
            world_normal,
            world_position: hit_point + sphere.center,
            hit_distance,
            index: object_index,
        }
    }
    pub fn miss(&self) -> Option<HitPayload> {
        None
    }
}
pub struct HitPayload {
    world_normal: Vec3,
    world_position: Vec3,
    hit_distance: f32,
    index: usize,
}
