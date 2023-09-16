use glam::Vec3;

use crate::{material::Material, sphere::Sphere, camera::Camera};
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub materials: Vec<Material>,
    pub camera: Camera,
}

impl Scene {
    pub fn example_scene() -> Scene {
        let spheres = vec![
            Sphere::new(-10. * Vec3::X, 2.5, 0),
            Sphere::new(Vec3::new(-2.8, -0.65, 1.7), 0.5, 1),
            Sphere::new(Vec3::new(- 3.7, -0.4, -0.05), 0.6, 2),
            Sphere::new(Vec3::new(-4.5, -0.1, -1.55), 0.7, 1),
            Sphere::new(Vec3::new(-6.0, 0.4, -0.0), 0.2, 1),
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
        let materials = vec![
            Material::new([0.75, 1.0, 1.0]),
            Material::new([1.0, 0.8, 1.0]),
            Material::new([1.0, 0.9, 0.9]),
            ];
        let camera = Camera::new(45., 0.1, 100., 1., 1.);

        Scene { spheres, materials, camera }
    }
}
