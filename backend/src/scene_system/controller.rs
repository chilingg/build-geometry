use crate::{
    data::prelude::*,
};

use winit::event::*;

pub struct Controller {
    pub cursor_state: CursorState,
    input_state: InputState,
    resize_event: Option<winit::dpi::PhysicalSize<u32>>
}

impl Controller {
    pub fn new() -> Self {
        Self {
            input_state: InputState {
                left: ElementState::Released,
                right: ElementState::Released,
                middle: ElementState::Released,
            },
            cursor_state: CursorState::default(),
            resize_event: None,
        }
    }

    pub fn window_resize(&self) -> Option<winit::dpi::PhysicalSize<u32>> {
        self.resize_event
    }

    pub fn point_from_screen(view_data: &ViewData, point: &winit::dpi::PhysicalPosition<f64>) -> WorldPoint {
        WorldPoint::new(
            (point.x as f32 - view_data.size.width / 2.0) * view_data.pixel_size + view_data.center.x,
            (view_data.size.height / 2.0 - point.y as f32) * view_data.pixel_size + view_data.center.y,
        )
    }

    pub fn update(&mut self, view_data: &mut DirtyFlag<ViewData>) {
        if self.input_state.middle == ElementState::Pressed {
            if let Some(moved) = self.cursor_state.moved.as_mut() {
                view_data.write().center -= *moved;
                self.cursor_state.pos -= *moved;
            }
        }

        self.cursor_state.moved = None;
        self.resize_event = None;
    }

    pub fn precess(&mut self, event: &WindowEvent, view_data: &mut DirtyFlag<ViewData>) -> bool {
        const SCALE_SPEED: f32 = 0.8;

        match event {
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, v),
                ..
            } => {
                view_data.write().pixel_size *= SCALE_SPEED.powi(*v as i32);
                true
            },
            WindowEvent::MouseInput {
                state,
                button,
                ..
            } => {
                match button {
                    MouseButton::Left => self.input_state.left = *state,
                    MouseButton::Right => self.input_state.right = *state,
                    MouseButton::Middle => self.input_state.middle = *state,
                    _ => return false
                }
                true
            },
            WindowEvent::CursorMoved {
                position,
                ..
            } => {
                let position = Self::point_from_screen(view_data.unchecked_read(), position);
                *self.cursor_state.moved.get_or_insert(WorldVector::zero()) += position - self.cursor_state.pos;
                self.cursor_state.pos = position;
                
                false
            },
            WindowEvent::Resized(physical_size) => {
                self.resize_event = Some(*physical_size);
                view_data.write().size = ScreenSize::new(physical_size.width as _, physical_size.height as _);
                false
            },
            _ => false
        }
    }
}

pub struct InputState {
    pub left: ElementState,
    pub right: ElementState,
    pub middle: ElementState,
}

#[derive(Default)]
pub struct CursorState {
    pub pos: WorldPoint,
    pub moved: Option<WorldVector>,
}
