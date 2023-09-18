use crate::math;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub seed: u32,
    pub bounces: u32,
    pub _offset: [f32; 2],
    pub sky_color: [f32; 4],
}
impl Default for Globals {
    fn default() -> Self {
        Self {
            seed: 0x5748,
            bounces: 8,
            _offset: [0.; 2],
            sky_color: math::as_rgbaf32(crate::COLORS[0]),
        }
    }
}
