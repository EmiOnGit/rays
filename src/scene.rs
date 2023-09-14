use glam::Vec3;
use image::Rgba;

use crate::renderer::{material::Material, sphere::Sphere};
#[derive(Default)]
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub materials: Vec<Material>,
}

impl Scene {
    pub fn example_scene() -> Scene {
        let sphere1 = Sphere::new(Vec3::ZERO, 0.5);
        let mut sphere2 = Sphere::new(Vec3::new(-1., -0.5, 3.), 1.);
        sphere2.albedo = Rgba([0.9, 0.0, 0.1, 1.]);
        Scene {
            spheres: vec![sphere1, sphere2],
            materials: Vec::new(),
        }
    }
}
