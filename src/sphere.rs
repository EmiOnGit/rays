use glam::Vec3;
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    /// Center of sphere in world space
    pub center: Vec3,
    /// Radius of the sphere
    pub radius: f32,
    /// Which material to use.
    /// panics if the index is not found.
    pub material_index: u32,
    // needed for shader alignment
    _offset: [f32; 3],
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material_index: usize) -> Sphere {
        Sphere {
            center,
            radius,
            material_index: material_index as u32,
            _offset: [0.; 3],
        }
    }
}
unsafe impl bytemuck::Pod for Sphere {}
unsafe impl bytemuck::Zeroable for Sphere {}
