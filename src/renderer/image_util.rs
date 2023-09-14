use image::Rgb32FImage;
use itertools::Itertools;
#[cfg(feature = "rayon")]
use rayon::prelude::IntoParallelIterator;
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
#[cfg(feature = "rayon")]
pub fn iter_mut_image_buffer(image_buffer: &mut Rgb32FImage) -> rayon::slice::IterMut<f32> {
    use rayon::prelude::IntoParallelRefMutIterator;

    image_buffer.par_iter_mut()
}
#[cfg(not(feature = "rayon"))]
pub fn iter_mut_image_buffer(image_buffer: &mut Rgb32FImage) -> std::slice::IterMut<f32> {
    image_buffer.iter_mut()
}
impl ImageSize {
    #[cfg(feature = "rayon")]
    pub fn into_iter(self) -> rayon::vec::IntoIter<(usize, usize)> {
        (0..self.height as usize)
            .cartesian_product(0..self.width as usize)
            .collect::<Vec<(usize, usize)>>()
            .into_par_iter()
    }
    #[cfg(not(feature = "rayon"))]
    pub fn into_iter(self) -> std::vec::IntoIter<(usize, usize)> {
        (0..self.height as usize)
            .cartesian_product(0..self.width as usize)
            .collect::<Vec<(usize, usize)>>()
            .into_iter()
    }
}
