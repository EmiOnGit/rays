use crate::math;

/// Global parameter that can be configured by the user
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    /// Seed for the random generation.
    /// random numbers are e.g. used for sampling on the accumodation buffer
    pub seed: u32,
    /// Amount of bounces a ray is able to do.
    /// Can drasticely increase the amount  of work the tracer has to do.
    pub bounces: u32,
    /// padding offset
    pub _offset: [f32; 2],
    /// Color of the sky. Since the sky gives ambient light to the objects, it also influences the scene feel overall
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
