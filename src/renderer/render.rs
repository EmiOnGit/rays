use crate::{
    camera::Camera,
    math::{self, ray::Ray},
    scene::Scene,
};
use glam::Vec3;
use image::{DynamicImage, Rgb};
#[cfg(feature = "rayon")]
use rayon::prelude::ParallelIterator;

use super::{image_util, Renderer};

impl Renderer {
    pub fn render(&mut self, camera: &Camera, scene: &Scene) {
        self.acc_frame = self.acc_frame + 1;
        let colors: Vec<Rgb<f32>> = self
            .size()
            .into_iter()
            .map(|(y, x)| self.per_pixel(x, y, camera, scene))
            .collect();

        for (i, pixel) in self.acc_buffer.pixels_mut().enumerate() {
            let c = colors[i];
            *pixel = [pixel.0[0] + c[0], pixel.0[1] + c[1], pixel.0[2] + c[2], pixel.0[3]].into();
        }
        
    }
    pub fn update_image_buffer(&mut self) {
        let mut i = self.acc_buffer.clone();
        image_util::iter_mut_image_buffer(&mut i).for_each(|pixel| {
            *pixel = *pixel / self.acc_frame as f32;
        });

        self.image_buffer = DynamicImage::from(i).into_rgba32f();
        self.seed = math::pcg_hash(self.seed);
    }
    pub fn per_pixel(&self, x: usize, y: usize, camera: &Camera, scene: &Scene) -> Rgb<f32> {
        let direction = camera.ray_directions[x + y * self.size().width as usize];
        let mut ray = Ray::new(camera.position, direction);
        let bounces = 8;
        let mut light = Vec3::ZERO;
        let mut contribution = Vec3::ONE;
        let mut bounced = bounces;
        for i in 0..bounces {
            let payload = self.trace_ray(ray.clone(), scene);
            let Some(payload) = payload else {
                let sky_color = Vec3::new(0.012, 0.012, 0.015);
                light += sky_color * contribution;
                bounced = i + 1;
                break;
            };
            let sphere = &scene.spheres[payload.index];
            let material = scene.material(sphere);
            ray.origin = payload.world_position - payload.world_normal * 0.0001;
            ray.direction = (math::in_unit_sphere(self.seed ^ payload.hit_distance.to_bits().wrapping_mul(payload.index as u32 * 11 + 10))
                + payload.world_normal)
                / 2.;
            // albedo.apply(|c| c  * contribution);
            // light += Vec3::from(albedo.0);
            contribution *= Vec3::from(material.albedo.0);
            light += material.get_emission();
        }
        let raw_c = (light / bounced as f32).to_array();
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
