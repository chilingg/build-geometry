use backend::scene_system::SceneSystem;

use std::time;

pub struct GameCtrl {
    pub open: bool,
    now: time::Instant,
}

impl GameCtrl {
    pub fn new() -> Self {
        Self {
            open: true,
            now: time::Instant::now(),
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, game: &mut SceneSystem) {
        egui::Window::new("Game controller")
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.label(format!("view center: ({:.3}, {:.3})", game.view_data().center.x, game.view_data().center.y));
                ui.label(format!("pixcel size: {:.3}", game.view_data().pixel_size));

                ui.label(format!("fps: {:.1}", 1000.0 / self.now.elapsed().as_millis() as f32));
                self.now = time::Instant::now();
            });
    }
}