use glam::Vec3;

use crate::{material::Material, sphere::Sphere, math::rand};
#[derive(Default)]
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub materials: Vec<Material>,
}

impl Scene {
    pub fn example_scene() -> Scene {
        let mut spheres = vec![
            Sphere::new(-22. * Vec3::Z + 10. * Vec3::X, 5., 0),
            Sphere::new(Vec3::new(-2.5, -1., -3.), 0.5, 1),
            Sphere::new(Vec3::new(1., 1000., 3.), 1000.0, 4),
        ];
        let p = 0x372f;
        let count = 100;
        for i in 0..count {
            let x = rand(i ^ p ) * 50. - 25.;
            let z = rand((i + count ) ^ p) * 30.;
            let radius = rand((i + 2 * count ) ^ p) * 1.;
            let center = Vec3::new(x, - radius,z);
            spheres.push(
                Sphere::new(center, radius, i as usize % 5 + 1)
            )
        }
        let materials = vec![
            Material::new([0.0, 1.0, 1.0], 0.01, 0.9, 100.0),
            Material::new([0.9, 0.9, 0.1], 0.4, 0.9, 0.3),
            Material::new([0.6, 0.2, 0.0], 1., 0.9, 0.5),
            Material::new([0.9, 0.9, 0.9], 1., 0.9, 0.8),
            Material::new([1.,1.,1.], 1., 0.9, 0.),
            Material::new([0.8, 0.7, 0.2], 1., 0.9, 0.),
        ];
        Scene { spheres, materials }
    }
    pub fn material(&self, sphere: &Sphere) -> &Material {
        &(&self.materials)[sphere.material_index]
    }
}
