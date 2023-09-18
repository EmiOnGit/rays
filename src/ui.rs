use egui::{Color32, Context, DragValue, RichText};
use egui_wgpu::renderer::ScreenDescriptor;
use egui_winit::State;

use wgpu::{CommandEncoder, Device, Queue, RenderPass, TextureFormat};
use winit::{event::WindowEvent, event_loop::EventLoopWindowTarget};

use crate::{globals::Globals, scene::Scene};

pub struct UiManager {
    egui_renderer: egui_wgpu::Renderer,
    egui_primitives: Vec<egui::ClippedPrimitive>,
    context: Context,
    screen_descriptor: ScreenDescriptor,
    state: State,
    header_color: Color32,
}
impl UiManager {
    pub fn new(
        device: &Device,
        surface_format: TextureFormat,
        screen_descriptor: ScreenDescriptor,
        event_loop: &EventLoopWindowTarget<()>,
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
            header_color: Color32::from_rgb(255, 150, 150),
        }
    }
    pub fn handle_window_event(&mut self, window_event: &WindowEvent) -> bool {
        self.state.on_event(&self.context, window_event).repaint
    }

    pub fn resize(&mut self, screen_descriptor: ScreenDescriptor) {
        self.screen_descriptor = screen_descriptor;
    }
    pub fn run(
        &mut self,
        device: &Device,
        queue: &Queue,
        window: &winit::window::Window,
        scene: &mut Scene,
        globals: &mut Globals,
    ) -> bool {
        let egui_raw_input = self.state.take_egui_input(window);
        let mut reset_renderer = false;
        let egui_full_output = self.context.run(egui_raw_input, |ctx| {
            egui::Window::new("Scene").vscroll(true).show(ctx, |ui| {
                ui.heading(RichText::new("Materials").color(self.header_color));

                for (i, material) in scene.materials.iter_mut().enumerate() {
                    let i_name = format!("{}", i);
                    let name = Scene::material_name(i).unwrap_or(&i_name);
                    ui.collapsing(name, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("color");
                            let color = &mut material.albedo.0;
                            reset_renderer |=
                                ui.color_edit_button_rgba_premultiplied(color).changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("emission");
                            let color = &mut material.emission;
                            reset_renderer |=
                                ui.color_edit_button_rgba_unmultiplied(color).changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("emission strength");
                            let color = &mut material.emission[3];
                            reset_renderer |= ui.add(DragValue::new(color).speed(0.01)).changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("metallic");
                            reset_renderer |= ui
                                .add(
                                    DragValue::new(&mut material.metallic)
                                        .speed(0.01)
                                        .clamp_range(0..=1),
                                )
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("specular_intensity");
                            reset_renderer |= ui
                                .add(
                                    DragValue::new(&mut material.specular_intensity)
                                        .speed(0.01)
                                        .clamp_range(0..=1),
                                )
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("roughness");
                            reset_renderer |= ui
                                .add(
                                    DragValue::new(&mut material.roughness)
                                        .speed(0.01)
                                        .clamp_range(0..=1),
                                )
                                .changed();
                        });
                    });
                }
                ui.add_space(10.);

                ui.heading(RichText::new("Spheres").color(self.header_color));
                for (i, sphere) in scene.spheres.iter_mut().enumerate() {
                    ui.collapsing(format!("Sphere {i}"), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("position");
                            reset_renderer |= ui
                                .add(DragValue::new(&mut sphere.center.x).speed(0.01))
                                .changed();
                            reset_renderer |= ui
                                .add(DragValue::new(&mut sphere.center.y).speed(0.01))
                                .changed();
                            reset_renderer |= ui
                                .add(DragValue::new(&mut sphere.center.z).speed(0.01))
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("radius");
                            reset_renderer |= ui
                                .add(DragValue::new(&mut sphere.radius).speed(0.001))
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("material index");
                            reset_renderer |= ui
                                .add(
                                    DragValue::new(&mut sphere.material_index)
                                        .clamp_range(0..=scene.materials.len() - 1),
                                )
                                .changed();
                        });
                    });
                }
            });

            egui::Window::new("Camera").show(ctx, |ui| {
                ui.label(format!("Transform"));
                ui.horizontal(|ui| {
                    reset_renderer |= ui
                        .add(DragValue::new(&mut scene.camera.position.x).speed(0.01))
                        .changed();
                    reset_renderer |= ui
                        .add(DragValue::new(&mut scene.camera.position.y).speed(0.01))
                        .changed();
                    reset_renderer |= ui
                        .add(DragValue::new(&mut scene.camera.position.z).speed(0.01))
                        .changed();
                });
                ui.label(format!("Fov"));
                ui.horizontal(|ui| {
                    reset_renderer |= ui
                        .add(DragValue::new(&mut scene.camera.fov).speed(0.01))
                        .changed();
                });
            });

            egui::Window::new("Globals").show(ctx, |ui| {
                ui.label(format!("bounces"));
                reset_renderer |= ui.add(DragValue::new(&mut globals.bounces)).changed();
                ui.label(format!("sky color"));
                let color = &mut globals.sky_color;
                reset_renderer |= ui.color_edit_button_rgba_unmultiplied(color).changed();
            });
        });

        for (id, image_delta) in egui_full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(device, queue, id, &image_delta);
        }
        self.egui_primitives = self.context.tessellate(egui_full_output.shapes);
        reset_renderer
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
