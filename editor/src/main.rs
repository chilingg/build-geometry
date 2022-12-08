use backend::app::App;
use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};

mod ui_system;

fn main() {
    backend::app::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_maximized(true)
        .build(&event_loop)
        .unwrap();
    let app = App::new(event_loop, window);
    let system = ui_system::UiSystem::new(&app);
    
    app.run(system);
}
