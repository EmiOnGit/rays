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
            Sphere::new(Vec3::Z * 2., 0.5, 0),
            // Sphere::new(Vec3::new(-2.5, -1., -3.), 0.5, 1),
            // Sphere::new(Vec3::new(1., 1000., 3.), 1000.0, 4),
        ];
        // let p = 0x372f;
        // let count = 100;
        // for i in 0..count {
        //     let x = rand(i ^ p) * 50. - 25.;
        //     let z = rand((i + count) ^ p) * 30.;
        //     let radius = rand((i + 2 * count) ^ p) * 1.;
        //     let center = Vec3::new(x, -radius, z);
        //     spheres.push(Sphere::new(center, radius, i as usize % 5 + 1))
        // }
        let materials = vec![Material::new([0.0, 1.0, 1.0])];
        Scene { spheres, materials }
    }
}
