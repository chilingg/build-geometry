use backend::scene_system::SceneSystem;

pub struct GameCtrl {
    pub open: bool,
}

impl GameCtrl {
    pub fn new() -> Self {
        Self { open: true }
    }

    pub fn ui(&mut self, ctx: &egui::Context, game: &mut SceneSystem) {
        egui::Window::new("Game controller")
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.label("The game system has not started!");
            });
    }
}