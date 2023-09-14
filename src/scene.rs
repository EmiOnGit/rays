use glam::Vec3;

use crate::{material::Material, sphere::Sphere};
#[derive(Default)]
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub materials: Vec<Material>,
}

impl Scene {
    pub fn example_scene() -> Scene {
        let spheres = vec![
            Sphere::new(Vec3::ZERO, 1., 0),
            Sphere::new(Vec3::new(-1., 0.5, -2.), 0.5, 1),
            Sphere::new(Vec3::new(1., -1., 3.), 2., 0),
            Sphere::new(Vec3::new(1., 101.5, 3.), 100., 2),
        ];
        let materials = vec![
            Material::new([0.9, 0.0, 0.1], 0.4, 0.9),
            Material::new([0.9, 0.5, 0.1], 0.01, 0.9),
            Material::new([0.2, 0.2, 0.2], 1., 0.9),
        ];
        Scene { spheres, materials }
    }
    pub fn material(&self, sphere: &Sphere) -> &Material {
        &(&self.materials)[sphere.material_index]
    }
}
