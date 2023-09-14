use rayon::prelude::{IntoParallelRefIterator, ParallelIterator, IndexedParallelIterator};

use crate::{camera::Camera, scene::Scene, math::ray::Ray};

use super::{Renderer, trace_ray, image_util::color_f32_to_u8};

impl Renderer {
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