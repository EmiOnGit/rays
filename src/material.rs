use image::Rgba;
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub albedo: Rgba<f32>,
    /// strength is encoded in alpha
    pub emission: Rgba<f32>,
}

unsafe impl bytemuck::Pod for Material {}
unsafe impl bytemuck::Zeroable for Material {}


impl Material {
    pub fn new(albedo: [f32; 3], emission_color: [f32; 3], emission_power: f32) -> Material {
        let albedo = [albedo[0], albedo[1], albedo[2], 1.];
        let emission = [emission_color[0], emission_color[1], emission_color[2], emission_power];
        Material {
            albedo: Rgba(albedo),
            emission: Rgba(emission),
        }
    }

}
