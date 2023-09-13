mod camera;
mod math;
mod render_time;
mod renderer;
mod scene;
mod app;

use camera::Camera;
use log::warn;
use render_time::RenderTimeDiagnostic;
use renderer::Renderer;
use scene::Scene;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    pollster::block_on(run());
}
pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = Renderer::new(window).await;
    let mut render_timer = RenderTimeDiagnostic::new();
    let mut count = 0;
    let mut camera = Camera::new(45., 0.1, 100.,state.image_buffer.width() as f32,state.image_buffer.height() as f32);
    let mut mouse_pressed = false;
    let mut scene = Scene::example_scene();
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update(&camera, &scene);
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
            let render_time = render_timer.increment();
            count = (count + 1) % 200;
            if count == 0 {
                warn!("render time: {:?} ms", render_time.0);
                warn!(
                    "avg render time: {:?} ms",
                    render_timer.avg_render_time().0
                );
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            state.window().request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() => {
            if state.input(event) {
                return;
            }
            match event {
                WindowEvent::MouseInput { button, state, .. } => {
                    if &MouseButton::Right == button {
                        match state {
                            ElementState::Pressed => mouse_pressed = true,
                            ElementState::Released => mouse_pressed = false,
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if mouse_pressed {
                        camera.on_rotate(position);
                    } else {
                        camera.last_mouse_position = None;
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                    camera.resize(physical_size.width, physical_size.height);

                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                }
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    camera.on_keyboard_event(input, render_timer.peak().0);
                }
                _ => {}
            }
        }
        _ => {}
    });
}
