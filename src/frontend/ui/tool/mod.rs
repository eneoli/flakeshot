use crate::frontend::screenshot_window::MouseEvent;

use super::drawable::Drawable;

pub mod crop;

pub enum ToolCommand {
    Nop,
    Crop(f64, f64, f64, f64),
}

pub trait Tool {
    fn handle_mouse_move(&mut self, x: f64, y: f64) -> ToolCommand;
    fn handle_mouse_press(&mut self, x: f64, y: f64) -> ToolCommand;
    fn handle_mouse_release(&mut self, x: f64, y: f64) -> ToolCommand;
    fn get_drawable(&self) -> &dyn Drawable;

    fn handle_mouse_event(&mut self, event: MouseEvent) -> ToolCommand {
        match event {
            MouseEvent::MouseMove(x, y) => self.handle_mouse_move(x, y),
            MouseEvent::MosePress(_, x, y) => self.handle_mouse_press(x, y),
            MouseEvent::MouseRelease(_, x, y) => self.handle_mouse_release(x, y),
        }
    }
}
