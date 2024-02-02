use std::{cell::RefCell, rc::Rc};

use anyhow::Context;
use cairo::ImageSurface;
use image::DynamicImage;

use super::{
    file_chooser::FileChooser,
    ui::{canvas::Canvas, toolbar::ToolbarEvent},
};

type RenderHandler = dyn Fn(&UiManager);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SaveDestination {
    File,
    Clipboard,
}

pub struct UiManager {
    canvas: Rc<RefCell<Canvas>>,
    on_render_handler: Vec<Box<RenderHandler>>,
}

impl UiManager {
    pub fn new(total_width: i32, total_height: i32) -> Self {
        UiManager {
            canvas: Rc::new(RefCell::new(
                Canvas::new(total_width, total_height).expect("Couldn't create canvas."),
            )),
            on_render_handler: vec![],
        }
    }

    pub fn stamp_image(&self, x: f64, y: f64, image: &DynamicImage) -> anyhow::Result<()> {
        self.canvas.borrow().stamp_image(x, y, image)?;

        self.notify_render_handler();

        Ok(())
    }

    pub fn crop(&self, x: f64, y: f64, width: i32, height: i32) -> anyhow::Result<ImageSurface> {
        self.canvas.borrow().crop(x, y, width, height)
    }

    pub fn handle_tool_event(&mut self, event: ToolbarEvent) -> anyhow::Result<&'static str> {
        match event {
            ToolbarEvent::SaveAsFile => {
                self.save_image(SaveDestination::File)?;
                Ok("Image successfully save to file")
            }
            ToolbarEvent::SaveIntoClipboard => {
                self.save_image(SaveDestination::Clipboard)?;
                Ok("Image saved to clipboard")
            }
            ToolbarEvent::Crop => {
                todo!()
            }
        }
    }

    pub fn on_render<F>(&mut self, handler: F)
    where
        F: Fn(&Self) + 'static,
    {
        self.on_render_handler.push(Box::new(handler));
    }

    fn notify_render_handler(&self) {
        for handler in &self.on_render_handler {
            handler(self);
        }
    }

    fn save_image(&self, dest: SaveDestination) -> anyhow::Result<()> {
        let canvas_ref = self.canvas.clone();

        let width = canvas_ref.borrow().width() as u32;
        let height = canvas_ref.borrow().height() as u32;

        let image = canvas_ref
            .borrow()
            .crop_to_image(0.0, 0.0, width, height)
            .context("Couldn't crop image")?;

        match dest {
            SaveDestination::File => {
                FileChooser::open(move |file| {
                    if let Some(path) = file {
                        image.save(path).expect("Couldn't save image.");
                    }
                });
            }
            SaveDestination::Clipboard => crate::backend::save_to_clipboard(image)?,
        };

        Ok(())
    }
}
