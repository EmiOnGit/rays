use image::Rgba;
use itertools::Itertools;
use rayon::{prelude::IntoParallelIterator, vec::IntoIter};
use wgpu::Extent3d;

pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}
impl ImageSize {
    pub fn new(width: u32, height: u32) -> ImageSize {
        ImageSize { width, height }
    }
}
impl From<ImageSize> for Extent3d {
    fn from(value: ImageSize) -> Self {
        Extent3d {
            width: value.width,
            height: value.height,
            depth_or_array_layers: 1,
        }
    }
}
pub fn color_f32_to_u8(c: Rgba<f32>) -> Rgba<u8> {
    Rgba([
        (c[0] * 255.) as u8,
        (c[1] * 255.) as u8,
        (c[2] * 255.) as u8,
        (c[3] * 255.) as u8,
    ])
}

impl ImageSize {
    pub fn into_par_iter(self) -> IntoIter<(usize, usize)> {
        (0..self.height as usize)
            .cartesian_product(0..self.width as usize)
            .collect::<Vec<(usize, usize)>>()
            .into_par_iter()
    }
}
