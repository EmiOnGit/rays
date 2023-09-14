mod app;
mod camera;
mod math;
mod render_time;
mod renderer;
mod scene;

use app::App;
use log::warn;
use render_time::RenderTimeDiagnostic;
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
    let mut app = App::new(window).await;
    let mut render_timer = RenderTimeDiagnostic::new();
    let mut count = 0;
    let mut mouse_pressed = false;
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == app.window().id() => {
            match app.prepare() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                // Err(wgpu::SurfaceError::Lost) => app.resize(app.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
            app.queue();
            let render_time = render_timer.increment();
            count = (count + 1) % 200;
            if count == 0 {
                warn!("render time: {:?} ms", render_time.0);
                warn!("avg render time: {:?} ms", render_timer.avg_render_time().0);
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.

            app.update();
            app.window().request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == app.window().id() => {
            if app.input(event) {
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
                        app.camera.on_rotate(position);
                    } else {
                        app.camera.last_mouse_position = None;
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    app.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    app.resize(**new_inner_size);
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
                    app.handle_keyboard_input(input);
                }
                _ => {}
            }
        }
        _ => {}
    });
}
