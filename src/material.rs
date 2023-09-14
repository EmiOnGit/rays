use image::Rgba;

pub struct Material {
    pub albedo: Rgba<f32>,
    pub roughness: f32,
    pub metallic: f32,
}
impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Rgba([1., 0., 1., 1.]),
            roughness: Default::default(),
            metallic: Default::default(),
        }
    }
}

impl Material {
    pub fn new(albedo: [f32; 3], roughness: f32, metallic: f32) -> Material {
        let albedo = [albedo[0], albedo[1], albedo[2], 1.];
        Material {
            albedo: Rgba(albedo),
            roughness,
            metallic,
        }
    }
}
