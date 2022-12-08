use backend::app::{ System, App, State };

use egui_wgpu::renderer::{
    RenderPass,
    ScreenDescriptor
};

mod style_editor;

pub struct UiSystem {
    state: egui_winit::State,
    ctx: egui::Context,
    rpass: egui_wgpu::renderer::RenderPass,
    style_editer: style_editor::StyleEditor,
    output_data: Option<(egui::TexturesDelta, Vec<egui::ClippedPrimitive>)>,
}

impl UiSystem {
    pub fn new<T>(app: &App<T>) -> Self {
        Self {
            state: egui_winit::State::new(&app.event_loop),
            ctx: egui::Context::default(),
            rpass: RenderPass::new(
                &app.state.device,
                app.state.config.format,
                1
            ),
            style_editer: style_editor::StyleEditor::new(),
            output_data: None,
        }
    }
}

impl System for UiSystem {
    fn start(&mut self, _state: &State) {
        self.ctx.set_style(style_editor::default_style());
    }

    fn precess(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.state.on_event(&self.ctx, event)
    }

    fn update(&mut self, state: &State) {
        // Begin to draw the UI frame.
        let raw_input = self.state.take_egui_input(&state.window);
        let full_output = self.ctx.run(raw_input, |ctx| {
            self.style_editer.ui(ctx);
        });

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let paint_jobs = self.ctx.tessellate(full_output.shapes);

        self.output_data = Some((full_output.textures_delta, paint_jobs));
        self.state.handle_platform_output(&state.window, &self.ctx, full_output.platform_output);
    }

    fn render(&mut self, state: &State, view: &wgpu::TextureView) {
        if let Some((textures_delta, paint_jobs)) = self.output_data.take() {
            let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });
    
            // Upload all resources for the GPU.
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [state.config.width, state.config.height],
                pixels_per_point: self.state.pixels_per_point(),
            };
            for (id, ref image_delta) in textures_delta.set {
                self.rpass.update_texture(&state.device, &state.queue, id, image_delta);
            }
            self.rpass.update_buffers(&state.device, &state.queue, &paint_jobs, &screen_descriptor);
    
            // Record all render passes.
            self.rpass.execute(
                &mut encoder,
                &view,
                &paint_jobs,
                &screen_descriptor,
                Some(wgpu::Color::BLACK),
            );
            // Submit the commands.
            state.queue.submit(std::iter::once(encoder.finish()));
    
            for id in &textures_delta.free {
                self.rpass.free_texture(id);
            }        
        }
    }
}