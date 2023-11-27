use glam::{Quat, Vec3, Vec4, Vec4Swizzles};

/// Returns a random Vec3 with values between 0 and 1
pub(crate) fn rand_vec(mut seed: u32) -> Vec3 {
    let seed1 = pcg_hash(seed);
    let seed2 = pcg_hash(seed1);
    seed = pcg_hash(seed2);
    let v1 = seed1 as f32 / u32::MAX as f32;
    let v2 = seed2 as f32 / u32::MAX as f32;
    let v3 = seed as f32 / u32::MAX as f32;
    Vec3::new(v1, v2, v3)
}

pub(crate) fn pcg_hash(seed: u32) -> u32 {
    let state = seed.wrapping_mul(747779605).wrapping_add(2891336453);
    let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277803737);

    (word >> 22) ^ word
}

pub(crate) fn cross(q1: Quat, q2: Quat) -> Quat {
    let q1 = Vec4::from(q1);
    let q2 = Vec4::from(q2);
    let w = q1.w * q2.w - q1.xyz().dot(q2.xyz());
    let x = q1.wxy().dot(q2.xwz()) - q1.z * q2.y;
    let y = q1.wyz().dot(q2.ywx()) - q1.x * q2.z;
    let z = q1.wzx().dot(q2.zwy()) - q1.y * q2.x;
    Quat::from_xyzw(x, y, z, w)
}

fn linear_f32_from_gamma_u8(s: u8) -> f32 {
    if s <= 10 {
        s as f32 / 3294.6
    } else {
        ((s as f32 + 14.025) / 269.025).powf(2.4)
    }
}
pub fn as_rgbf32(rgb8: [u8; 3]) -> [f32; 3] {
    let r = linear_f32_from_gamma_u8(rgb8[0]);
    let g = linear_f32_from_gamma_u8(rgb8[1]);
    let b = linear_f32_from_gamma_u8(rgb8[2]);
    [r, g, b]
}
pub fn as_rgbaf32(rgb8: [u8; 3]) -> [f32; 4] {
    let r = linear_f32_from_gamma_u8(rgb8[0]);
    let g = linear_f32_from_gamma_u8(rgb8[1]);
    let b = linear_f32_from_gamma_u8(rgb8[2]);
    [r, g, b, 1.]
}
