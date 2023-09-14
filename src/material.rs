use glam::Vec3;
use image::Rgb;

pub struct Material {
    pub albedo: Rgb<f32>,
    pub roughness: f32,
    pub metallic: f32,
    pub emission_color: Vec3,
    pub emission_power: f32,
}
impl Default for Material {
    fn default() -> Self {
        let r = 204 as f32 / 255.;
        let g = 128 as f32 / 255.;
        let b = 51 as f32 / 255.;
        Self {
            albedo: Rgb([1., 0.6, 0.6]),
            roughness: Default::default(),
            metallic: Default::default(),
            emission_color: Vec3::new(r, g, b),
            emission_power: 1.,
        }
    }
}

impl Material {
    pub fn new(albedo: [f32; 3], roughness: f32, metallic: f32, emission_power: f32) -> Material {
        let albedo = [albedo[0], albedo[1], albedo[2]];
        Material {
            albedo: Rgb(albedo),
            roughness,
            metallic,
            emission_power,
            ..Default::default()
        }
    }
    pub fn get_emission(&self) -> Vec3 {
        self.emission_power * self.emission_color
    }
}
