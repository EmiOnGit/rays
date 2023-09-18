use image::Rgba;
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub metallic: f32,
    pub specular_intensity: f32,
    pub roughness: f32,
    // between 0 and 1
    pub fog: f32,
    // pub specular: f32,
    pub albedo: Rgba<f32>,
    /// strength is encoded in alpha
    pub emission: [f32; 4],
}

unsafe impl bytemuck::Pod for Material {}
unsafe impl bytemuck::Zeroable for Material {}

impl Material {
    pub fn new() -> Material {
        Material {
            albedo: Rgba([1., 1., 1., 1.]),
            emission: [1., 1., 1., 0.],
            metallic: 0.,
            specular_intensity: 1.,
            roughness: 0.1,
            fog: 0.,
        }
    }
    /// Defaults to white
    pub fn with_color(mut self, albedo: [f32; 3]) -> Self {
        let albedo = [albedo[0], albedo[1], albedo[2], 1.];
        self.albedo = Rgba(albedo);
        self
    }
    /// Defaults to no emission
    pub fn with_emission(mut self, color: [f32; 3], strength: f32) -> Self {
        let emission = [color[0], color[1], color[2], strength];
        let albedo = [color[0], color[1], color[2], 1.];

        self.emission = emission;
        self.albedo = Rgba(albedo);
        self
    }
    // pub fn with_metallic(mut self, metallic: f32) -> Self {
    //     self.metallic = metallic;
    //     self
    // }
    /// Defaults to 1.
    pub fn with_specular_intensity(mut self, specular_intensity: f32) -> Self {
        self.specular_intensity = specular_intensity;
        self
    }
    /// Defaults to 0.1
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness;
        self
    }
}
