use backend::{
    app::{
        System,
        App,
        State,
    },
    scene_system::{
        SceneSystem,
        Scene,
    },
    renderer::DefaultRenderer,
    data::prelude::*,
};

use egui_wgpu::renderer::{
    RenderPass,
    ScreenDescriptor
};

mod style_editor;
mod game_ctrl;

pub struct UiSystem {
    state: egui_winit::State,
    ctx: egui::Context,
    rpass: egui_wgpu::renderer::RenderPass,

    style_editer: style_editor::StyleEditor,
    game_ctrl: game_ctrl::GameCtrl,

    output_data: Option<(egui::TexturesDelta, Vec<egui::ClippedPrimitive>)>,

    game: Option<SceneSystem>,
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
            game_ctrl: game_ctrl::GameCtrl::new(),
            output_data: None,
            game: None,
        }
    }
}

impl System for UiSystem {
    fn start(&mut self, state: &State) {
        self.ctx.set_style(style_editor::default_style());

        let mut game = SceneSystem::new(
            Scene{
                graph: vec![
                    GraphType::Circle { center: WorldPoint::new(0.0, 0.0), radius: 200.0 },
                    GraphType::Circle { center: WorldPoint::new(0.0, 300.0), radius: 200.0 },
                    ],
                tip: vec![],
            },
            DefaultRenderer::new(state)
        );
        game.start(state);
        self.game = Some(game);
    }

    fn precess(&mut self, event: &winit::event::WindowEvent) -> bool {
        use winit::event::*;

        if let Some(game) = &mut self.game {
            if self.state.on_event(&self.ctx, event) {
                true
            } else if game.precess(event) {
                true
            } else {
                match event {
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::F12),
                            state: ElementState::Pressed,
                            ..
                        },
                        ..
                    } => { 
                        self.style_editer.open = !self.style_editer.open;
                        true
                    },
                    _ => false,
                }
            }
        } else {
            false
        }
    }

    fn update(&mut self, state: &State) {
        if let Some(game) = &mut self.game {
            // Begin to draw the UI frame.
            let raw_input = self.state.take_egui_input(&state.window);
            let full_output = self.ctx.run(raw_input, |ctx| {
                self.style_editer.ui(ctx);
                self.game_ctrl.ui(ctx, game);
            });
    
            // End the UI frame. We could now handle the output and draw the UI with the backend.
            let paint_jobs = self.ctx.tessellate(full_output.shapes);
    
            self.output_data = Some((full_output.textures_delta, paint_jobs));
            self.state.handle_platform_output(&state.window, &self.ctx, full_output.platform_output);
    
            // Game update
            game.update(state);
        }
    }

    fn render(&mut self, state: &State, view: &wgpu::TextureView) {
        if let Some(game) = &mut self.game {
            game.render(state, view);
            
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
                    None,
                );
                // Submit the commands.
                state.queue.submit(std::iter::once(encoder.finish()));
        
                for id in &textures_delta.free {
                    self.rpass.free_texture(id);
                }        
            }
        }
    }
}