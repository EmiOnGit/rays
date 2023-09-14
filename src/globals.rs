
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Globals {
    pub seed: u32,
}

unsafe impl bytemuck::Pod for Globals {}
unsafe impl bytemuck::Zeroable for Globals {}