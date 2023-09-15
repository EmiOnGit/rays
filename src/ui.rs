use egui::Context;
use egui_wgpu::renderer::ScreenDescriptor;
use wgpu::{Device, TextureFormat, Queue, CommandEncoder, RenderPass};

pub struct UiManager {
        // --egui
        egui_renderer: egui_wgpu::Renderer,
        egui_primitives: Vec<egui::ClippedPrimitive>,
        context: Context,
        screen_descriptor: ScreenDescriptor,
}
impl UiManager {
    pub fn new(device: &Device, surface_format: TextureFormat, screen_descriptor: ScreenDescriptor) -> Self {
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1);
        let context = Context::default();

        UiManager {
            egui_renderer,
            egui_primitives: Vec::new(),
            context,
            screen_descriptor,
        }
    }
    pub fn run(&mut self, device: &Device, queue: &Queue) {
        let egui_raw_input = egui::RawInput::default();
        let egui_full_output = self.context.run(egui_raw_input, |ctx| {
            egui::Area::new("area").default_pos(&[100.,100.]).show(ctx, |ui| {
                ui.label("test label");
            });
        });
        for (id,image_delta) in egui_full_output.textures_delta.set {
            self.egui_renderer.update_texture(device, queue, id, &image_delta);
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
    pub fn render<'a>(&'a mut self, render_pass: & mut RenderPass<'a>) {
        self.egui_renderer
        .render(render_pass, &self.egui_primitives, &self.screen_descriptor);
    }
}