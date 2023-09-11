mod renderer;
mod render_time;

use log::info;
use render_time::RenderTimeDiagnostic;
use renderer::State;
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
    let mut state = State::new(window).await;
    let mut render_time_logger = RenderTimeDiagnostic::new();
    let mut count = 0;
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {

            state.update();
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
            let render_time = render_time_logger.increment();
            count = (count + 1) % 200;
            if count == 0 {
                info!("render time: {:?} ms", render_time.0);
                info!("avg render time: {:?} ms", render_time_logger.avg_render_time().0);
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
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
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
                _ => {}
            }
        }
        _ => {}
    });
}
