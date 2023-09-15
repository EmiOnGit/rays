use image::Rgba;
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub albedo: Rgba<f32>,
    // pub roughness: f32,
    // pub metallic: f32,
    // pub emission_color: Vec3,
    // pub emission_power: f32,
}

unsafe impl bytemuck::Pod for Material {}
unsafe impl bytemuck::Zeroable for Material {}
impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Rgba([1., 0.6, 0.6, 1.]),
        }
    }
}

impl Material {
    pub fn new(albedo: [f32; 3]) -> Material {
        let albedo = [albedo[0], albedo[1], albedo[2], 1.];
        Material {
            albedo: Rgba(albedo),
        }
    }
    // pub fn get_emission(&self) -> Vec3 {
    //     self.emission_power * self.emission_color
    // }
}
