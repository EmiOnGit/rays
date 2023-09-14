use glam::{Quat, Vec4, Vec4Swizzles, Vec3};

pub mod ray;
/// Returns a random float between 0 and 1
pub fn rand(mut seed: u32) -> f32 {
    seed = pcg_hash(seed);
    seed as f32 / u32::MAX as f32
}

pub fn pcg_hash(seed: u32) -> u32 {
    let state = seed.wrapping_mul(747779605).wrapping_add(2891336453);
    let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277803737);

    (word >> 22) ^ word
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
pub fn in_unit_sphere(seed: u32) -> Vec3{
    let seed2 = pcg_hash(seed);
    let x = rand(seed);
    let y = rand(seed ^ seed2);
    let z = rand(seed2);
    Vec3::new(x * 2. - 1., y * 2. - 1., z * 2. - 1.).normalize()
}