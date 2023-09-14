use image::Rgb;

pub struct Material {
    pub albedo: Rgb<f32>,
    pub roughness: f32,
    pub metallic: f32,
}
impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Rgb([1., 0., 1.]),
            roughness: Default::default(),
            metallic: Default::default(),
        }
    }
}

impl Material {
    pub fn new(albedo: [f32; 3], roughness: f32, metallic: f32) -> Material {
        let albedo = [albedo[0], albedo[1], albedo[2]];
        Material {
            albedo: Rgb(albedo),
            roughness,
            metallic,
        }
    }
}
