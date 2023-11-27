use image::Rgba;
/// Represents the material of a rendered sphere.
/// Multiple Spheres can use the same material without a problem.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub metallic: f32,
    pub specular_intensity: f32,
    /// A roughness of 0 corresponds to a perfectly flat surface, like a surface
    pub roughness: f32,
    // between 0 and 1
    pub fog: f32,
    /// The color before applying any of the other settings.
    pub albedo: Rgba<f32>,
    /// Strength of the emmision is encoded in alpha.
    /// Therefore if alpha is set to 0, no emission is added to the material
    pub emission: [f32; 4],
}

/// We need to make this claim to be able to send the material to the shader.
/// In this case we mainly have to be aware of the 0-padding restrictions
unsafe impl bytemuck::Pod for Material {}
unsafe impl bytemuck::Zeroable for Material {}

impl Material {
    /// Creates a default material
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
