use glam::Vec3;

use crate::{
    camera::Camera,
    material::Material,
    math::{self, rand_vec},
    sphere::Sphere,
};
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub materials: Vec<Material>,
    pub camera: Camera,
}

impl Scene {
    /// Generates an example scene.
    /// This is currently the default
    pub fn example_scene() -> Scene {
        let mut spheres = vec![
            Sphere::new(Vec3::new(0., 1000., 0.), 1000.4, 0),
            Sphere::new(Vec3::new(100., -100., -200.), 50.4, 1),
            Sphere::new(Vec3::new(-20.8, -4.57, 10.7), 5., 4),
            Sphere::new(Vec3::new(10.22, -6.4, -20.25), 7., 5),
        ];
        for x in 0..10 {
            for y in 0..10 {
                for z in 1..3 {
                    let mut f = rand_vec(x + y * 0x43182 + z * 0x13457);
                    let r = rand_vec((f.x * 100000.) as u32 + x);

                    let material_index = ((r.length() * 0x483290 as f32) as usize ^ 0x587439) % 5;
                    let radius = f.y * 1.2;
                    f.y *= -3. * z as f32;
                    f.x *= 50.;
                    f.z *= 50.;
                    f.x -= 25.;
                    f.z -= 25.;
                    spheres.push(Sphere::new(f, radius, material_index + 2));
                }
            }
        }

        let palette: Vec<[f32; 3]> = crate::COLORS.into_iter().map(math::as_rgbf32).collect();
        let materials = vec![
            Material::new() // ground
                .with_color(palette[1])
                .with_roughness(0.15),
            Material::new() // sun
                .with_emission(palette[2], 100.)
                .with_roughness(0.4),
            Material::new() // light
                .with_emission(palette[2], 2.)
                .with_roughness(0.4),
            Material::new().with_color(palette[3]),
            Material::new().with_color(palette[4]),
            Material::new().with_color(palette[5]).with_roughness(0.5),
            Material::new().with_color(palette[6]).with_roughness(0.8),
        ];
        let camera = Camera::new(100., 0.1, 100., 1., 1.);

        Scene {
            spheres,
            materials,
            camera,
        }
    }
    pub fn material_name(index: usize) -> Option<&'static str> {
        match index {
            0 => Some("Ground"),
            1 => Some("Sun"),
            2 => Some("Light"),
            _ => None,
        }
    }
}
