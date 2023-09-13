use glam::{Quat, Vec4, Vec4Swizzles};
use image::Rgba;

pub mod ray;
/// Returns a random float between 0 and 1
pub fn rand(mut seed: u32) -> f32 {
    seed = pcg_hash(seed);
    seed as f32 / u32::MAX as f32
}

fn pcg_hash(seed: u32) -> u32 {
    let state = seed * 747779605 + 2891336453;
    let word = ((state >> ((state >> 28) + 4)) ^ state) * 277803737;

    (word >> 22) ^ word
}
pub fn color_f32_to_u8(c: Rgba<f32>) -> Rgba<u8> {
    Rgba([
        (c[0] * 255.) as u8,
        (c[1] * 255.) as u8,
        (c[2] * 255.) as u8,
        (c[3] * 255.) as u8,
    ])
}
pub fn cross(q1: Quat, q2: Quat) -> Quat {
    let q1 = Vec4::from(q1);
    let q2 = Vec4::from(q2);
    let w = q1.w * q2.w - q1.xyz().dot(q2.xyz());
    let x = q1.wxy().dot(q2.xwz()) - q1.z * q2.y;
    let y = q1.wyz().dot(q2.ywx()) - q1.x * q2.z;
    let z = q1.wzx().dot(q2.zwy()) - q1.y * q2.x;
    Quat::from_xyzw(x, y, z, w)
}