use build_geometry::game_system::GameSystem;

pub struct GameCtrl {
    pub open: bool,
}

impl GameCtrl {
    pub fn new() -> Self {
        Self { open: true }
    }

    pub fn ui(&mut self, ctx: &egui::Context, game: &mut GameSystem) {
        egui::Window::new("Game Ctrl")
            .open(&mut self.open)
            .show(ctx, |ui| {
                match game.data() {
                    Some(game) => {
                        let (bezier_uniform, dirty) = &mut game.bezier_uniform;
                        ui.horizontal(|ui| {
                            *dirty |= ui.add(egui::Slider::new(&mut game.tolerance, 0.1..=15.0)).changed();
                            ui.label("Tolerance");
                        });
                        ui.horizontal(|ui| {
                            *dirty |= ui.add(egui::Slider::new(
                                &mut bezier_uniform.segment_size,
                                1..=game.bezier_segment.approximate_length(game.tolerance) as u32
                            )).changed();
                            ui.label("Segment size");
                        });
                        ui.horizontal(|ui| {
                            *dirty |= ui.add(egui::Slider::new(
                                &mut bezier_uniform.stroke_width,
                                1.0..=30.0
                            )).changed();
                            ui.label("Stroke size");
                        });
                    },
                    _ => { ui.label("The game system has not started!"); },
                }
            });
    }
}