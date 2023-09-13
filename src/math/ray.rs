use glam::Vec3;

pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Ray { origin, direction }
    }
    pub fn at(&self, lambda: f32) -> Vec3 {
        self.origin + lambda * self.direction
    }
}