use glam::{Quat, Vec3, Vec4};
use image::{Pixel, Rgba};
use rayon::prelude::{IndexedParallelIterator, ParallelIterator};

use crate::{
    camera::Camera,
    math::{self, ray::Ray},
    scene::Scene,
};

use super::{image_util, Renderer};

impl Renderer {
    pub fn per_pixel(&self, x: usize, y: usize, camera: &Camera, scene: &Scene) -> Rgba<f32> {
        let direction = camera.ray_directions[x + y * self.size().width as usize];
        let mut ray = Ray::new(camera.position, direction);
        let bounces = 5;
        let mut color_acc = Vec4::ZERO;
        let mut bounced = bounces;
        for i in 0..bounces {
            let payload = self.trace_ray(ray.clone(), scene);
            let Some(payload) = payload else {
                bounced = i;
                break;
            };
            let sphere = &scene.spheres[payload.index];

            let mat = scene.material(sphere);
            ray.origin = payload.world_position - payload.world_normal * 0.0001;
            let r = Quat::from_rotation_arc(
                ray.direction.normalize(),
                payload.world_normal.normalize()
                    + mat.roughness * math::rand(payload.hit_distance.to_bits() + self.seed),
            );
            ray.direction = r * ray.direction.normalize();
            let light_dir = Vec3::new(1., 1., 1.).normalize();
            let light_intensity = payload.world_normal.dot(-light_dir).max(0.);
            let mut color = scene.material(sphere).albedo;
            color.apply_without_alpha(|c| c * light_intensity);
            color_acc += Vec4::from(color.0);
        }
        let mut raw_c = (color_acc / bounced as f32).to_array();
        raw_c[3] = 1.;
        let color = Rgba(raw_c);
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
    pub fn render(&mut self, camera: &Camera, scene: &Scene, dt: f32) {
        let mut colors = Vec::with_capacity(camera.ray_directions.len());
        self.size()
            .into_par_iter()
            .map(|(y, x)| {
                let color = self.per_pixel(x, y, camera, scene);
                image_util::color_f32_to_u8(color)
            })
            .collect_into_vec(&mut colors);
        for (i, pixel) in self.image_buffer.pixels_mut().enumerate() {
            *pixel = colors[i];
        }
        self.seed = self.seed.wrapping_add(dt.to_bits());
    }
}
pub struct HitPayload {
    world_normal: Vec3,
    world_position: Vec3,
    hit_distance: f32,
    index: usize,
}
