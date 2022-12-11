use backend::app::App;
use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};

mod game_system;
use game_system::GameSystem;

fn main() {
    backend::app::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        // .with_maximized(true)
        .with_inner_size( winit::dpi::PhysicalSize {
                width: 500,
                height: 500,
        })
        .build(&event_loop)
        .unwrap();

    App::new(event_loop, window).run(GameSystem::new());
}
