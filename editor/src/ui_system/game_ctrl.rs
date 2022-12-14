use build_geometry::game_system::GameSystem;

pub struct GameCtrl {
    pub open: bool,
}

impl GameCtrl {
    pub fn new() -> Self {
        Self { open: true }
    }

    pub fn ui(&mut self, ctx: &egui::Context, game: &mut GameSystem) {
        egui::Window::new("Game controller")
            .open(&mut self.open)
            .show(ctx, |ui| {
                match &mut game.data {
                    Some(game) => {
                        use build_geometry::game_system::BezierUniform;

                        let (bezier_data, dirty) = game.bezier_data.get_all();

                        ui.label("Bezier info");
                        ui.label(format!("subsegment per {} pixels", BezierUniform::PER_PIXELS));
                        ui.label(format!("tolerance: {}", BezierUniform::TOLERANCE));
                        ui.label(format!("subdivide: {}", bezier_data.subdivide));
                        ui.label(format!("approximate length: {}", bezier_data.segment.approximate_length(BezierUniform::TOLERANCE)));

                        ui.horizontal(|ui| {
                            *dirty |= ui.add(egui::Slider::new(
                                &mut bezier_data.stroke_width,
                                0.5..=50.0
                            )).changed();
                            ui.label("Stroke size");
                        });
                    },
                    _ => { ui.label("The game system has not started!"); },
                }
            });
    }
}