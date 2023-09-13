use glam::{Vec3, Mat4, Vec2, Quat, Vec4, Vec4Swizzles};

use rayon::prelude::{IntoParallelIterator, ParallelIterator, IndexedParallelIterator};
use winit::{event::{KeyboardInput, VirtualKeyCode}, dpi::PhysicalPosition};

use crate::math;

#[derive(Debug)]
pub struct Camera {
    projection: Mat4,
    view: Mat4,
    inverse_projection: Mat4,
    inverse_view: Mat4,
    // vertical fov
    fov: f32,
    near_clip: f32,
    far_clip: f32,
    pub position: Vec3,
    forward: Vec3,
    pub ray_directions: Vec<Vec3>,
    viewport_height: f32,
    viewport_width: f32,

    pub last_mouse_position: Option<PhysicalPosition<f64>>,
}

impl Camera {
    pub fn new(fov: f32, near_clip: f32, far_clip: f32, viewport_width: f32, viewport_height: f32) -> Camera {
        let forward = Vec3::Z;
        let position = -3.* Vec3::Z;

        let mut camera = Camera {
            forward,
            position,
            fov,
            near_clip,
            far_clip,
            projection: Mat4::default(),
            inverse_projection: Mat4::default(),
            view: Mat4::default(),
            inverse_view: Mat4::default(),
            viewport_height,
            viewport_width,
            ray_directions: Vec::new(),
            last_mouse_position: None
        };
        camera.recalculate_view();
        camera.recalculate_projection();
        camera.calculate_ray_directions();
        camera
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == self.viewport_width as u32 && height == self.viewport_height as u32 {
            return
        }
        self.viewport_width = width as f32;
        self.viewport_height = height as f32;
        self.recalculate_projection();
        self.recalculate_view();

        self.calculate_ray_directions();
    }
    pub fn on_keyboard_event(&mut self, input: &KeyboardInput, dt: f32) {
        let speed = 5. * dt;
        let up = Vec3::Y;
	    let right_direction = self.forward.cross(up);
        match input.virtual_keycode {
            Some(VirtualKeyCode::W) => self.position += self.forward * speed,
            Some(VirtualKeyCode::S) => self.position -= self.forward * speed,
            Some(VirtualKeyCode::A) => self.position -= right_direction * speed,
            Some(VirtualKeyCode::D) => self.position += right_direction * speed,
            Some(VirtualKeyCode::Q) => self.position -= up * speed,
            Some(VirtualKeyCode::E) => self.position += up * speed,
            _ => {return}
        }
        self.recalculate_view();
        self.calculate_ray_directions();
    }
    pub fn on_rotate(&mut self, mouse_position: &PhysicalPosition<f64>) {
        let right_direction = self.forward.cross(Vec3::Y);
        match self.last_mouse_position {
            Some(last) => {
                let last = Vec2::new(last.x as f32, last.y as f32);
                let current = Vec2::new(mouse_position.x as f32, mouse_position.y as f32);
                let delta = 0.006 * (current - last);

                let q1 = Quat::from_axis_angle(right_direction, -delta.y);
                let q2 = Quat::from_axis_angle(Vec3::Y, -delta.x);
                let q = math::cross(q1, q2).normalize();
                self.forward = q * self.forward;
                self.recalculate_view();
                self.calculate_ray_directions();
                self.last_mouse_position = Some(mouse_position.clone());
            },
            None => self.last_mouse_position = Some(mouse_position.clone()),

        }
    }
    fn calculate_ray_directions(&mut self) {
        // for (y,x) in  (0..self.viewport_height as usize).cartesian_product(0..self.viewport_width as usize).collect::<Vec<(usize,usize)>>().into_par_iter(){
        let height = self.viewport_height as usize;
        let width = self.viewport_width as usize;
        (0..height * width).into_par_iter().map(|i| {
            let x = i % width;
            let y = i / width;
            let mut coord = Vec2::new(x as f32 / self.viewport_width, y as f32 / self.viewport_height);
            coord = coord * 2. - Vec2::ONE; 
            let target = self.inverse_projection * Vec4::new(coord.x, coord.y, 1.,1.);
            let target = (target.xyz() / target.w).normalize();
            let direction = (self.inverse_view * Vec4::new(target.x, target.y, target.z, 0.)).xyz();
            direction
        }).collect_into_vec(&mut self.ray_directions)
        
    }
    fn recalculate_view(&mut self) {
        self.view = Mat4::look_at_rh(self.position, self.position + self.forward, Vec3::Y);
        self.inverse_view = self.view.inverse();
    }
    fn recalculate_projection(&mut self) {
        let fov = self.fov.to_radians();
        self.projection = Mat4::perspective_rh(fov, self.viewport_height / self.viewport_width , self.near_clip, self.far_clip);
        self.inverse_projection = self.projection.inverse();
    }
    
}