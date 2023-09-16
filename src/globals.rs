
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub seed: u32,
    pub bounces: u32,
    pub _offset: [f32; 2],
    pub sky_color: [f32; 4],
}