use glam::{Mat4, Vec2};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Globals {
    pub seed: u32,
    pub viewport: Vec2,
    pub inverse_projection: Mat4,
    pub inverse_view: Mat4,
}

unsafe impl bytemuck::Pod for Globals {}
unsafe impl bytemuck::Zeroable for Globals {}
