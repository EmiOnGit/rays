use glam::Vec3;
use image::Rgba;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub albedo: Rgba<f32>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Sphere {
        Sphere {
            center,
            radius,
            albedo: Rgba([1., 0., 1., 1.]),
        }
    }
}
