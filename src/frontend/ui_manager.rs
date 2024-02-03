use cairo::ImageSurface;
use image::DynamicImage;

use super::{
    file_chooser::FileChooser,
    rectangle::Rectangle,
    screenshot_window::MouseEvent,
    tool_manager::ToolManager,
    ui::{
        canvas::{Canvas, CanvasDrawable},
        drawable::Drawable,
        tool::ToolCommand,
        toolbar::ToolbarEvent,
    },
};

type RenderObserver = dyn Fn(&UiManager);

enum CanvasDrawableStrategy<'a> {
    DrawActive(&'a dyn Drawable),
    DrawInactive(&'a dyn Drawable),
    DrawFinal(&'a dyn Drawable),
}

impl<'a> CanvasDrawable for CanvasDrawableStrategy<'a> {
    fn draw(&self, ctx: &cairo::Context, surface: &ImageSurface) {
        match self {
            CanvasDrawableStrategy::DrawActive(drawable) => drawable.draw_active(ctx, surface),
            CanvasDrawableStrategy::DrawInactive(drawable) => drawable.draw_inactive(ctx, surface),
            CanvasDrawableStrategy::DrawFinal(drawable) => drawable.draw_final(ctx, surface),
        }
    }
}

pub struct UiManager {
    tool_manager: ToolManager,
    canvas: Canvas,
    selection: Rectangle,
    drawables: Vec<Box<dyn Drawable>>,
    render_observer: Vec<Box<RenderObserver>>,
}

impl UiManager {
    pub fn new(total_width: i32, total_height: i32) -> Self {
        UiManager {
            tool_manager: ToolManager::new(),
            canvas: Canvas::new(total_width, total_height).expect("Couldn't create canvas."),
            selection: Rectangle::with_size(total_width as f64, total_height as f64),
            drawables: vec![],
            render_observer: vec![],
        }
    }

    pub fn on_render<F>(&mut self, handler: F)
    where
        F: Fn(&Self) + 'static,
    {
        self.render_observer.push(Box::new(handler));
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
        self.notify_render_observer();

        Ok(())
    }

    pub fn crop(&self, x: f64, y: f64, width: i32, height: i32) -> anyhow::Result<ImageSurface> {
        self.canvas.crop(x, y, width, height)
    }

    pub fn add_drawable(&mut self, drawable: Box<dyn Drawable>) {
        self.drawables.push(drawable)
    }

    pub fn handle_tool_event(&mut self, event: ToolbarEvent) {
        match event {
            ToolbarEvent::SaveAsFile => self.save_canvas_to_file(),
            ToolbarEvent::SaveIntoClipboard => {}
            ToolbarEvent::ToolSelect(tool_identifier) => {
                self.tool_manager.set_active_tool(Some(tool_identifier))
            }
        }
    }

    pub fn handle_mouse_event(&mut self, event: MouseEvent) {
        if let Some(tool) = self.tool_manager.active_tool_mut() {
            let cmd = tool.handle_mouse_event(event);
            self.handle_tool_command(cmd);
            self.render();
        }
    }

    fn set_crop_area(&mut self, rectangle: Rectangle) {
        self.selection = rectangle;
    }

    fn handle_tool_command(&mut self, command: ToolCommand) {
        match command {
            ToolCommand::Crop(rectangle) => self.set_crop_area(rectangle),
            ToolCommand::Noop => {}
        }
    }

    fn render(&mut self) {
        self.canvas.clear().expect("Couldn't clear canvas.");

        for drawable in &self.drawables {
            self.canvas
                .render_drawable(&CanvasDrawableStrategy::DrawInactive(drawable.as_ref()));
        }

        if let Some(tool) = self.tool_manager.active_tool() {
            self.canvas
                .render_drawable(&CanvasDrawableStrategy::DrawActive(tool.get_drawable()));
        }

        self.notify_render_observer();
    }

    fn notify_render_observer(&self) {
        for observer in &self.render_observer {
            observer(self);
        }
    }

    fn render_screenshot(&self) -> Canvas {
        let mut canvas = self.canvas.from_original();

        for drawable in &self.drawables {
            canvas.render_drawable(&CanvasDrawableStrategy::DrawFinal(drawable.as_ref()));
        }

        if let Some(tool) = self.tool_manager.active_tool() {
            canvas.render_drawable(&CanvasDrawableStrategy::DrawFinal(tool.get_drawable()));
        }

        canvas
    }

    fn save_canvas_to_file(&self) {
        let canvas = self.render_screenshot();

        let Rectangle { x1, x2, y1, y2 } = self.selection;

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
