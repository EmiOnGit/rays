use glam::{Mat4, Quat, Vec2, Vec3};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{KeyboardInput, VirtualKeyCode},
};

use crate::math;

#[derive(Debug)]
pub struct Camera {
    /// Position in world space of the camera
    pub position: Vec3,
    /// Field of view corresponds to how much we can see.
    /// In this implementation the horizontal fov is used
    pub fov: f32,
    /// forward view direction of the camera
    forward: Vec3,
    // perspective projection
    viewport_height: f32,
    viewport_width: f32,
    /// nearest object distance that is still visible
    near_clip: f32,
    /// farest object distance that is still visible
    far_clip: f32,

    pub last_mouse_position: Option<PhysicalPosition<f64>>,
}

impl Camera {
    pub fn new(
        fov: f32,
        near_clip: f32,
        far_clip: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Camera {
        let forward = -Vec3::X;
        // #TODO no reason to hardcode a position
        let position = Vec3::new(77.7, -7.4, 10.);

        let camera = Camera {
            forward,
            position,
            fov,
            near_clip,
            far_clip,
            viewport_height,
            viewport_width,
            last_mouse_position: None,
        };
        camera
    }
    /// In case of a resize event of the viewport.
    /// This can happen whenever the window itself get resized
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let width = new_size.width;
        let height = new_size.height;
        if width == self.viewport_width as u32 && height == self.viewport_height as u32 {
            return;
        }
        self.viewport_width = width as f32;
        self.viewport_height = height as f32;
    }
    /// Handles keyboard events that let the camera move
    pub fn on_keyboard_event(&mut self, input: &KeyboardInput, dt: f32) -> bool {
        let speed = 30. * dt;
        let up = Vec3::Y;
        let right_direction = self.forward.cross(up);
        match input.virtual_keycode {
            Some(VirtualKeyCode::W) => self.position += self.forward * speed,
            Some(VirtualKeyCode::S) => self.position -= self.forward * speed,
            Some(VirtualKeyCode::A) => self.position -= right_direction * speed,
            Some(VirtualKeyCode::D) => self.position += right_direction * speed,
            Some(VirtualKeyCode::Q) => self.position -= up * speed,
            Some(VirtualKeyCode::E) => self.position += up * speed,
            _ => return false,
        }
        true
    }
    /// Handles mouse drags, which are used to rotate the camera
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

                self.last_mouse_position = Some(*mouse_position);
            }
            None => self.last_mouse_position = Some(*mouse_position),
        }
    }

    fn inverse_view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.forward, Vec3::Y).inverse()
    }
    fn inverse_projection_matrix(&self) -> Mat4 {
        let fov = self.fov.to_radians();

        Mat4::perspective_rh(
            fov,
            self.viewport_width / self.viewport_height,
            self.near_clip,
            self.far_clip,
        )
        .inverse()
    }
}

/// Camera representation for the gpu
///
/// The offsets are needed for the wgpu layout of structs and are not used
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    fov: [f32; 2],
    viewport: [f32; 2],
    camera_position: [f32; 4],
    // offset needed for padding restrictions
    _offset1: [f32; 8],
    inverse_projection: [f32; 16],
    inverse_view: [f32; 16],
    _offset2: [f32; 16],
}

impl From<&Camera> for CameraUniform {
    fn from(cam: &Camera) -> Self {
        let viewport = [cam.viewport_width, cam.viewport_height];
        let camera_position = cam.position.extend(0.).to_array();
        let inverse_projection = cam.inverse_projection_matrix().to_cols_array();
        let inverse_view = cam.inverse_view_matrix().to_cols_array();
        Self {
            fov: [cam.fov; 2],
            viewport,
            _offset1: [0.; 8],
            camera_position,
            inverse_projection,
            inverse_view,
            _offset2: [0.; 16],
        }
    }
}
