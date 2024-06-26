use std::{
    io::{Cursor, Write},
    process::Stdio,
};

use anyhow::Context;
use derive_where::derive_where;
use gtk4::cairo::{self, ImageSurface};
use image::{DynamicImage, GenericImageView, ImageFormat};
use notify_rust::Urgency;
use relm4::ComponentSender;

use crate::{
    config::Config,
    frontend::{
        shape::rectangle::Rectangle,
        window::{
            file_chooser::FileChooser,
            main_window::{AppModel, Command},
            notification::Notification,
            screenshot_window::MouseEvent,
        },
    },
};

use super::{
    canvas::{Canvas, CanvasDrawable},
    drawable::Drawable,
    tool::ToolCommand,
    tool_manager::ToolManager,
    toolbar::ToolbarEvent,
};

type RenderObserver = dyn Fn(&UiManager);

enum CanvasDrawableStrategy<'a> {
    Active(&'a dyn Drawable),
    Inactive(&'a dyn Drawable),
    Final(&'a dyn Drawable),
}

impl<'a> CanvasDrawable for CanvasDrawableStrategy<'a> {
    fn draw(&self, ctx: &cairo::Context, surface: &ImageSurface) {
        match self {
            CanvasDrawableStrategy::Active(drawable) => drawable.draw_active(ctx, surface),
            CanvasDrawableStrategy::Inactive(drawable) => drawable.draw_inactive(ctx, surface),
            CanvasDrawableStrategy::Final(drawable) => drawable.draw_final(ctx, surface),
        }
    }
}

#[derive_where(Debug)]
pub struct UiManager {
    tool_manager: ToolManager,
    canvas: Canvas,
    selection: Rectangle,
    drawables: Vec<Box<dyn Drawable>>,
    sender: ComponentSender<AppModel>,

    #[derive_where(skip(Debug))]
    render_observer: Vec<Box<RenderObserver>>,

    config: Config,
}

impl UiManager {
    pub fn new(
        total_width: i32,
        total_height: i32,
        sender: ComponentSender<AppModel>,
        config: Config,
    ) -> Self {
        UiManager {
            tool_manager: ToolManager::new(),
            canvas: Canvas::new(total_width, total_height).expect("Couldn't create canvas."),
            selection: Rectangle::with_size(total_width as f64, total_height as f64),
            drawables: vec![],
            render_observer: vec![],
            sender,
            config,
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
            ToolbarEvent::SaveAsFile => self.save_to_file(),
            ToolbarEvent::SaveIntoClipboard => {
                match self.save_to_clipboard() {
                    Ok(()) => self.sender.spawn_oneshot_command(|| Command::Close),
                    Err(err) => notify(
                        &self.sender,
                        Notification {
                            msg: err.to_string(),
                            urgency: Urgency::Critical,
                        },
                    ),
                };
            }
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
                .render_drawable(&CanvasDrawableStrategy::Inactive(drawable.as_ref()));
        }

        if let Some(tool) = self.tool_manager.active_tool() {
            self.canvas
                .render_drawable(&CanvasDrawableStrategy::Active(tool.get_drawable()));
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
            canvas.render_drawable(&CanvasDrawableStrategy::Final(drawable.as_ref()));
        }

        if let Some(tool) = self.tool_manager.active_tool() {
            canvas.render_drawable(&CanvasDrawableStrategy::Final(tool.get_drawable()));
        }

        canvas
    }
}

impl UiManager {
    fn get_crop_image(&self) -> DynamicImage {
        let canvas = self.render_screenshot();
        let Rectangle { fst, snd } = self.selection;

        canvas
            .crop_to_image(
                fst.x,
                fst.y,
                std::cmp::max(0, (snd.x - fst.x).floor() as u32),
                std::cmp::max(0, (snd.y - fst.y).floor() as u32),
            )
            .expect("Couldn't crop canvas.")
    }

    fn save_to_file(&self) {
        let img = self.get_crop_image();

        let sender = self.sender.clone();
        FileChooser::open(move |file| {
            if let Some(path) = file {
                match img.save(&path) {
                    Ok(()) => notify(
                        &sender,
                        Notification {
                            msg: format!("Screenshot save to {}", path.to_string_lossy()),
                            urgency: Urgency::Low,
                        },
                    ),
                    Err(err) => notify(
                        &sender,
                        Notification {
                            msg: format!(
                                "Couldn't save screenshot to {}: {}",
                                path.to_string_lossy(),
                                err
                            ),
                            urgency: Urgency::Critical,
                        },
                    ),
                };
            }
        });
    }

    fn save_to_clipboard(&self) -> anyhow::Result<()> {
        let img = self.get_crop_image();

        let mut child = {
            let (clip_man, args) = if crate::backend::is_wayland() {
                (
                    self.config.wayland.clipboard.cmd(),
                    self.config.wayland.clipboard.args(),
                )
            } else {
                (
                    self.config.x11.clipboard.cmd(),
                    self.config.x11.clipboard.args(),
                )
            };

            std::process::Command::new(clip_man)
                .args(args)
                .stdin(Stdio::piped())
                .spawn()
                .unwrap_or_else(|err| {
                    panic!(
                        "Coulnd't spawn '{}'. Is it a typo or did you really install it?\n{}",
                        &clip_man, err
                    )
                })
        };

        let mut image_bytes: Vec<u8> = {
            let dim = img.dimensions();
            Vec::with_capacity((dim.0 * dim.1) as usize)
        };

        img.write_to(&mut Cursor::new(&mut image_bytes), ImageFormat::Png)
            .context("Couldn't write image to stdin of clipboard process")?;

        let child_stdin = child
            .stdin
            .as_mut()
            .context("Couldn't get stdin of clipboard-process")?;
        child_stdin
            .write_all(&image_bytes)
            .context("Couldn't write image bytes into clipboard")?;
        child_stdin
            .flush()
            .context("Couldn't flush image to clipboard.")?;

        notify(
            &self.sender,
            Notification {
                msg: "Screenshot saved to clipboard.".to_string(),
                urgency: Urgency::Low,
            },
        );

        Ok(())
    }
}

fn notify(app_model_sender: &ComponentSender<AppModel>, msg: Notification) {
    app_model_sender.spawn_oneshot_command(|| Command::Notify(msg))
}
