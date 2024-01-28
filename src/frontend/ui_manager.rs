use cairo::ImageSurface;
use image::DynamicImage;

use super::{
    file_chooser::FileChooser,
    screenshot_window::MouseEvent,
    ui::{
        canvas::Canvas,
        drawable::Drawable,
        tool::{crop::Crop, Tool, ToolCommand},
        toolbar::ToolbarEvent,
    },
};

type RenderHandler = dyn Fn(&UiManager);

pub struct UiManager {
    canvas: Canvas,
    drawables: Vec<Box<dyn Drawable>>,
    on_render_handler: Vec<Box<RenderHandler>>,
    active_tool: Option<Box<dyn Tool>>,
    selected_x1: f64,
    selected_x2: f64,
    selected_y1: f64,
    selected_y2: f64,
}

impl UiManager {
    pub fn new(total_width: i32, total_height: i32) -> Self {
        UiManager {
            canvas: Canvas::new(total_width, total_height).expect("Couldn't create canvas."),
            drawables: vec![],
            on_render_handler: vec![],
            active_tool: Some(Box::new(Crop::new())),
            selected_x1: 0.0,
            selected_x2: total_width as f64,
            selected_y1: 0.0,
            selected_y2: total_height as f64,
        }
    }

    pub fn persist_canvas(&mut self) {
        self.canvas.save().expect("Couldn't persist canvas.");
    }

    pub fn stamp_image(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        image: &DynamicImage,
    ) -> anyhow::Result<()> {
        self.canvas.stamp_image(x, y, width, height, image)?;
        self.notify_render_handler();

        Ok(())
    }

    pub fn crop(&self, x: f64, y: f64, width: i32, height: i32) -> anyhow::Result<ImageSurface> {
        self.canvas.crop(x, y, width, height)
    }

    pub fn handle_tool_event(&mut self, event: ToolbarEvent) {
        match event {
            ToolbarEvent::SaveAsFile => self.save_canvas_to_file(),
            ToolbarEvent::SaveIntoClipboard => {}
            ToolbarEvent::Crop => {}
        }
    }

    pub fn handle_mouse_event(&mut self, event: MouseEvent) {
        if let Some(tool) = &mut self.active_tool {
            let cmd = tool.handle_mouse_event(event);
            self.handle_tool_command(cmd);
            self.render();
        }
    }

    fn handle_tool_command(&mut self, command: ToolCommand) {
        match command {
            ToolCommand::Crop(x1, x2, y1, y2) => self.set_crop_area(x1, x2, y1, y2),
            ToolCommand::Nop => {}
        }
    }

    pub fn set_crop_area(&mut self, x1: f64, x2: f64, y1: f64, y2: f64) {
        self.selected_x1 = x1;
        self.selected_x2 = x2;
        self.selected_y1 = y1;
        self.selected_y2 = y2;
    }

    pub fn add_drawable(&mut self, drawable: Box<dyn Drawable>) {
        self.drawables.push(drawable)
    }

    pub fn render(&mut self) {
        self.canvas.clear().expect("Couldn't clear canvas.");

        for drawable in &self.drawables {
            self.canvas.render_drawable(drawable.as_ref());
        }

        if let Some(tool) = &self.active_tool {
            self.canvas.render_drawable(tool.get_drawable());
        }

        self.notify_render_handler();
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

    fn render_screenshot(&self) -> Canvas {
        let mut canvas = self.canvas.from_original();

        for drawable in &self.drawables {
            canvas.render_drawable_final(drawable.as_ref());
        }

        if let Some(tool) = &self.active_tool {
            canvas.render_drawable_final(tool.get_drawable());
        }

        canvas
    }

    fn save_canvas_to_file(&self) {
        let canvas = self.render_screenshot();

        let x1 = self.selected_x1;
        let x2 = self.selected_x2;
        let y1 = self.selected_y1;
        let y2 = self.selected_y2;

        FileChooser::open(move |file| {
            if let Some(path) = file {
                canvas
                    .crop_to_image(
                        x1,
                        y1,
                        std::cmp::max(0, (x2 - x1).floor() as u32),
                        std::cmp::max(0, (y2 - y1).floor() as u32),
                    )
                    .expect("Couldn't crop canvas.")
                    .save(path)
                    .expect("Couldn't save image.");
            }
        });
    }
}
