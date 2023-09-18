mod app;
mod camera;
mod globals;
pub mod material;
mod math;
mod renderer;
mod scene;
pub mod sphere;
mod timer;
mod ui;

use app::App;
use simple_logger::SimpleLogger;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
const COLORS: [[u8; 3]; 7] = [
    [244, 244, 244], // sky
    [121, 135, 119], // ground
    [255, 217, 102], // light
    [189, 210, 182],
    [162, 178, 159],
    [244, 177, 131],
    [223, 166, 123],
];
fn main() {
    pollster::block_on(run());
}
pub async fn run() {
    SimpleLogger::default()
        .with_level(log::LevelFilter::Info)
        .with_module_level("wgpu_core", log::LevelFilter::Warn)
        .with_module_level("wgpu_hal", log::LevelFilter::Warn)
        .init()
        .unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut app = App::new(window, &event_loop).await;
    let mut mouse_pressed = false;
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == app.window().id() => {
            match app.prepare() {
                Ok(_) => {}
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
            app.queue();
            app.render_ui();
        }
        Event::MainEventsCleared => {
            app.update();
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            app.window().request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == app.window().id() => {
            app.handle_window_event(event);
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
                        app.scene.camera.on_rotate(position);
                        app.clear_renderer();
                    } else {
                        app.scene.camera.last_mouse_position = None;
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    app.resize(*physical_size, None);
                }
                WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    scale_factor,
                } => {
                    app.resize(**new_inner_size, Some(*scale_factor as f32));
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
