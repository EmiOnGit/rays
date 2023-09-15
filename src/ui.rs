use egui::Context;
use egui_wgpu::renderer::ScreenDescriptor;
use egui_winit::State;
use log::warn;
use wgpu::{CommandEncoder, Device, Queue, RenderPass, TextureFormat};
use winit::{event::WindowEvent, event_loop::EventLoopWindowTarget};

pub struct UiManager {
    // --egui
    egui_renderer: egui_wgpu::Renderer,
    egui_primitives: Vec<egui::ClippedPrimitive>,
    context: Context,
    screen_descriptor: ScreenDescriptor,
    state: State,
}
impl UiManager {
    pub fn new<T>(
        device: &Device,
        surface_format: TextureFormat,
        screen_descriptor: ScreenDescriptor,
        event_loop: &EventLoopWindowTarget<T>,
    ) -> Self {
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1);
        let context = Context::default();
        let state = State::new(&event_loop);

        UiManager {
            egui_renderer,
            egui_primitives: Vec::new(),
            context,
            screen_descriptor,
            state,
        }
    }
    pub fn handle_window_event(&mut self, window_event: &WindowEvent) -> bool {
        self.state.on_event(&self.context, window_event).repaint
    }

    pub fn resize(&mut self, screen_descriptor: ScreenDescriptor) {
        self.screen_descriptor = screen_descriptor;
    }
    pub fn run(&mut self, device: &Device, queue: &Queue, window: &winit::window::Window) {
        let egui_raw_input = self.state.take_egui_input(window);
        let egui_full_output = self.context.run(egui_raw_input, |ctx| {
            egui::Window::new("area")
                .default_height(500.)
                .show(ctx, |ui| {
                    ui.label("test label");
                    if ui.button("click me").hovered() {
                        warn!("hover on button");
                    }
                });
        });
        for (id, image_delta) in egui_full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(device, queue, id, &image_delta);
        }
        self.egui_primitives = self.context.tessellate(egui_full_output.shapes);
    }
    pub fn update_buffers(&mut self, encoder: &mut CommandEncoder, device: &Device, queue: &Queue) {
        let _commands = self.egui_renderer.update_buffers(
            &device,
            &queue,
            encoder,
            &self.egui_primitives,
            &self.screen_descriptor,
        );
    }
    pub fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>) {
        self.egui_renderer
            .render(render_pass, &self.egui_primitives, &self.screen_descriptor);
    }
}
