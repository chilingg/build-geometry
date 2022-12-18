use winit::event::*;

use crate:: {
    app::{ System, State },
    data::prelude::*,
    renderer::{ self, Renderer }
};

mod controller;
use controller::Controller;

mod scene;
pub use scene::Scene;

pub struct SceneSystemStruct {
}

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
}

impl System for SceneSystem {
    fn start(&mut self, state: &State) {
        let view_data = renderer::gen_view_data(state);
        self.renderer.update_view(&view_data, state);
        *self.view_data.write() = view_data;

        self.renderer.start_in_scene(&self.scene, self.view_data.unchecked_read().pixel_size, state);
    }

    fn update(&mut self, state: &State) {
        if let Some(size) = self.ctrl.window_resize() {
            self.renderer.resize(size, state);
        }

        self.ctrl.update(&mut self.view_data);

        if let (view_data, true) = self.view_data.get_all() {
            self.renderer.update_view(view_data, state);
            self.view_data.clean_flag();
        }

        self.renderer.update_scene(&self.scene, self.view_data.read().pixel_size, state);
    }

    fn precess(&mut self, event: &WindowEvent) -> bool {
        self.ctrl.precess(event, &mut self.view_data)
    }

    fn render(&mut self, state: &State, view: &wgpu::TextureView) {
        self.renderer.render(state, view);
    }
}