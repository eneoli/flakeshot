use std::{rc::Rc, cell::RefCell};

use crate::frontend::ui::drawable::Drawable;

use super::Tool;

pub struct Crop {
    is_active: bool,
    drawable: Rc<RefCell<Box<CropDrawable>>>,
}

impl Crop {
    pub fn new() -> Self {
        Self {
            is_active: false,
            drawable: Rc::new(RefCell::new(Box::new(CropDrawable::new()))),
        }
    }
}

impl Tool for Crop {
    fn handle_mouse_move(&mut self, x: f64, y: f64) {
        if self.is_active {
            self.drawable.borrow_mut().x2 = x;
            self.drawable.borrow_mut().y2 = y;
        }
    }

    fn handle_mouse_press(&mut self, x: f64, y: f64) {
        self.is_active = true;
        self.drawable.borrow_mut().x1 = x;
        self.drawable.borrow_mut().x2 = x;
        self.drawable.borrow_mut().y1 = y;
        self.drawable.borrow_mut().y2 = y;
    }

    fn handle_mouse_release(&mut self, x: f64, y: f64) {
        self.is_active = false;
        self.drawable.borrow_mut().x2 = x;
        self.drawable.borrow_mut().y2 = y;
    }

    fn get_drawable(&self) -> Rc<RefCell<Box<dyn Drawable>>> {
        self.drawable.clone() as Rc<RefCell<Box<dyn Drawable>>>
    }
}

struct CropDrawable {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl CropDrawable {
    pub fn new() -> Self {
        CropDrawable {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
        }
    }
}

impl Drawable for CropDrawable {
    fn draw(&self, ctx: &cairo::Context) {
        ctx.set_source_rgba(0.0, 0.0, 0.0, 0.5);
        ctx.rectangle(self.x1, self.y1, self.x2 - self.x1, self.y2 - self.y1);
        ctx.fill();
    }
}
