use crate::frontend::{shape::rectangle::Rectangle, window::screenshot_window::MouseEvent};

use super::drawable::Drawable;

pub mod crop;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ToolIdentifier {
    Crop,
}

pub enum ToolCommand {
    Noop,
    Crop(Rectangle),
}

pub trait Tool {
    fn handle_mouse_move(&mut self, x: f64, y: f64) -> ToolCommand;
    fn handle_mouse_press(&mut self, x: f64, y: f64) -> ToolCommand;
    fn handle_mouse_release(&mut self, x: f64, y: f64) -> ToolCommand;
    fn get_drawable(&self) -> &dyn Drawable;

    fn handle_mouse_event(&mut self, event: MouseEvent) -> ToolCommand {
        match event {
            MouseEvent::MouseMove { x, y } => self.handle_mouse_move(x, y),
            MouseEvent::MosePress { x, y, .. } => self.handle_mouse_press(x, y),
            MouseEvent::MouseRelease { x, y, .. } => self.handle_mouse_release(x, y),
        }
    }
}
