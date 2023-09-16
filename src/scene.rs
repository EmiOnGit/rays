use glam::Vec3;

use crate::{material::Material, sphere::Sphere, camera::Camera, math::rand_vec};
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub materials: Vec<Material>,
    pub camera: Camera,
}

impl Scene {
    pub fn example_scene() -> Scene {
        let mut spheres = vec![
            Sphere::new(Vec3::new(0.,10000.,0.), 10000.1, 0),
            Sphere::new(Vec3::new(-20.8, -4.57, 10.7), 5., 1),
            Sphere::new(Vec3::new(-10.5, -18.05, -50.35), 20., 2),
            Sphere::new(Vec3::new(10.22, -6.4, -20.25), 7., 1),
            Sphere::new(Vec3::new(5.55, -7.4, 0.0), 8., 1),
        ];
        for x in 0..20 {
            for y in 0..20 {
                let mut f = rand_vec(x + y * 0x4382);
                let radius =  f.y;
                f.y *= -1.;
                f.x *= 50.;
                f.z *= 50.;
                f.x -= 25.;
                f.z -= 25.;
                spheres.push(Sphere::new(f, radius, 0));
                
            }
        }
        let red_glow = [0.9,0.9,0.9];
        let materials = vec![
            Material::new([0.75, 1.0, 1.0], red_glow, 0.0),
            Material::new([1.0, 0.8, 1.0], red_glow, 0.),
            Material::new([0.4, 0.35, 0.35], red_glow, 1000.),
            ];
        let camera = Camera::new(45., 0.1, 100., 1., 1.);

        Scene { spheres, materials, camera }
    }
}
