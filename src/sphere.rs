use glam::Vec3;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material_index: usize,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material_index: usize) -> Sphere {
        Sphere {
            center,
            radius,
            material_index,
        }
    }
}
