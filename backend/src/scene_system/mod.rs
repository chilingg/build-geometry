use winit::event::*;

use crate:: {
    app::{ System, State },
    data::prelude::*,
    renderer::Renderer,
};

mod controller;
use controller::{ Controller, CursorState };

mod scene;
pub use scene::Scene;

pub struct SceneSystem {
    view_data: DirtyFlag<ViewData>,
    ctrl: Controller,
    scene: Scene,
    renderer: Box<dyn Renderer>,
}

impl SceneSystem {
    pub fn new<T: Renderer + 'static>(scene: Scene, renderer: T) -> Self {
        Self {
            view_data: DirtyFlag::new(ViewData::default()),
            ctrl: Controller::new(),
            scene,
            renderer: Box::new(renderer)
        }
    }

    pub fn view_data(&self) -> &ViewData {
        self.view_data.unchecked_read()
    }

    pub fn cursor_data(&self) -> &CursorState {
        &self.ctrl.cursor_state
    }
}

impl System for SceneSystem {
    fn start(&mut self, state: &State) {
        self.renderer.init_in_scene(&self.scene, state);
    }

    fn update(&mut self, state: &State) {
        if let (view_data, true) = self.view_data.get_all() {
            if self.ctrl.window_resize().is_some() {
                self.renderer.update_view_in_resize(view_data, state);
            } else {
                self.renderer.update_view(view_data, state);
            }
            self.view_data.clean_flag();
        }

        self.renderer.update_scene(&self.scene, state);

        self.ctrl.update(&mut self.view_data);
    }

    fn precess(&mut self, event: &WindowEvent) -> bool {
        self.ctrl.precess(event, &mut self.view_data)
    }

    fn render(&mut self, state: &State, output: &wgpu::SurfaceTexture) {
        self.renderer.render(state, output);
    }
}