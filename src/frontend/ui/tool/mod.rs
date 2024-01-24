use std::{rc::Rc, cell::RefCell};

use crate::frontend::screenshot_window::MouseEvent;

use super::drawable::Drawable;

pub mod crop;

pub trait Tool {
    fn handle_mouse_move(&mut self, x: f64, y: f64);
    fn handle_mouse_press(&mut self, x: f64, y: f64);
    fn handle_mouse_release(&mut self, x: f64, y: f64);
    fn get_drawable(&self) -> Rc<RefCell<Box<dyn Drawable>>>;

    fn handle_mouse_event(&mut self, event: MouseEvent) {
        match event {
            MouseEvent::MouseMove(x, y) => self.handle_mouse_move(x, y),
            MouseEvent::MosePress(_, x, y) => self.handle_mouse_press(x, y),
            MouseEvent::MouseRelease(_, x, y) => self.handle_mouse_release(x, y),
        }
    }
}
