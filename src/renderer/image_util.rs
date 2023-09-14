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

impl ImageSize {
    pub fn into_par_iter(self) -> IntoIter<(usize, usize)> {
        (0..self.height as usize)
            .cartesian_product(0..self.width as usize)
            .collect::<Vec<(usize, usize)>>()
            .into_par_iter()
    }
}
