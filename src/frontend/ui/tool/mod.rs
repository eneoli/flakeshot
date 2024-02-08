use crate::frontend::{
    shape::{point::Point, rectangle::Rectangle},
    window::screenshot_window::MouseEvent,
};

use super::drawable::Drawable;

pub mod crop;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolIdentifier {
    Crop,
}

#[derive(Debug)]
pub enum ToolCommand {
    Noop,
    Crop(Rectangle),
}

pub trait Tool: std::fmt::Debug {
    fn handle_mouse_move(&mut self, point: Point) -> ToolCommand;
    fn handle_mouse_press(&mut self, point: Point) -> ToolCommand;
    fn handle_mouse_release(&mut self, point: Point) -> ToolCommand;
    fn get_drawable(&self) -> &dyn Drawable;

    fn handle_mouse_event(&mut self, event: MouseEvent) -> ToolCommand {
        match event {
            MouseEvent::MouseMove(position) => self.handle_mouse_move(position),
            MouseEvent::MosePress { position, .. } => self.handle_mouse_press(position),
            MouseEvent::MouseRelease { position, .. } => self.handle_mouse_release(position),
        }
    }
}
