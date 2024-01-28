use cairo::ImageSurface;

use crate::frontend::ui::drawable::Drawable;

use super::{Tool, ToolCommand};

enum ControlPoint {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

const CONTROL_POINT_RADIUS: f64 = 7.5;
const CONTROL_POINT_TOLERANCE: f64 = 5.0;

pub struct Crop {
    is_active: bool,
    drawable: CropDrawable,
    is_within: bool,
    is_dragging: bool,
    mouse_x: f64,
    mouse_y: f64,
}

impl Crop {
    pub fn new() -> Self {
        Self {
            is_active: false,
            is_within: false,
            is_dragging: false,
            drawable: CropDrawable::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
        }
    }
}

impl Tool for Crop {
    fn handle_mouse_move(&mut self, x: f64, y: f64) -> ToolCommand {
        if self.is_active && !self.is_dragging {
            self.drawable.x2 = x;
            self.drawable.y2 = y;
        }

        if self.is_within && self.is_dragging {
            // move
            let delta_x = x - self.mouse_x;
            let delta_y = y - self.mouse_y;

            self.drawable.x1 += delta_x;
            self.drawable.x2 += delta_x;
            self.drawable.y1 += delta_y;
            self.drawable.y2 += delta_y;
        }

        self.is_within = self.drawable.is_within_selection(x, y);

        self.mouse_x = x;
        self.mouse_y = y;

        ToolCommand::Nop
    }

    fn handle_mouse_press(&mut self, x: f64, y: f64) -> ToolCommand {
        if !self.is_within {
            self.is_active = true;

            let control_point = self.drawable.is_within_control_point(x, y);

            if let Some(control_point) = control_point {
                let x_left = f64::min(self.drawable.x1, self.drawable.x2);
                let x_right = f64::max(self.drawable.x1, self.drawable.x2);
                let y_top = f64::min(self.drawable.y1, self.drawable.y2);
                let y_bottom = f64::max(self.drawable.y1, self.drawable.y2);

                // Extend current box
                match control_point {
                    ControlPoint::TopLeft => {
                        self.drawable.x1 = x_right;
                        self.drawable.y1 = y_bottom;
                    }
                    ControlPoint::TopRight => {
                        self.drawable.x1 = x_left;
                        self.drawable.y1 = y_bottom;
                    }
                    ControlPoint::BottomLeft => {
                        self.drawable.x1 = x_right;
                        self.drawable.y1 = y_top;
                    }
                    ControlPoint::BottomRight => {
                        self.drawable.x1 = x_left;
                        self.drawable.y1 = y_top;
                    }
                }

                self.drawable.x2 = x;
                self.drawable.y2 = y;
            } else {
                // Create new box
                self.drawable.x1 = x;
                self.drawable.y1 = y;
                self.drawable.x2 = x;
                self.drawable.y2 = y;
            }

            self.is_dragging = false;
        } else {
            self.is_dragging = true;
        }

        ToolCommand::Nop
    }

    fn handle_mouse_release(&mut self, x: f64, y: f64) -> ToolCommand {
        self.is_active = false;
        self.is_dragging = false;

        if !self.is_within {
            self.drawable.x2 = x;
            self.drawable.y2 = y;
        }

        ToolCommand::Crop(
            self.drawable.x1,
            self.drawable.x2,
            self.drawable.y1,
            self.drawable.y2,
        )
    }

    fn get_drawable(&self) -> &dyn Drawable {
        &self.drawable
    }
}

pub struct CropDrawable {
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

    pub fn is_within_selection(&self, x: f64, y: f64) -> bool {
        let x_left = f64::min(self.x1, self.x2);
        let x_right = f64::max(self.x1, self.x2);
        let y_top = f64::min(self.y1, self.y2);
        let y_bottom = f64::max(self.y1, self.y2);

        x_left <= x && x_right >= x && y_top <= y && y_bottom >= y && self.is_within_control_point(x, y).is_none()
    }

    pub fn is_within_control_point(&self, x: f64, y: f64) -> Option<ControlPoint> {
        let is_within_point = |xc: f64, yc: f64, x: f64, y: f64| {
            let radius = CONTROL_POINT_RADIUS + CONTROL_POINT_TOLERANCE;
            xc - radius <= x && xc + radius >= x && yc - radius <= y && yc + radius >= y
        };

        let x_left = f64::min(self.x1, self.x2);
        let x_right = f64::max(self.x1, self.x2);
        let y_top = f64::min(self.y1, self.y2);
        let y_bottom = f64::max(self.y1, self.y2);

        // Top Left
        if is_within_point(x_left, y_top, x, y) {
            return Some(ControlPoint::TopLeft);
        }

        // Top Right
        if is_within_point(x_right, y_top, x, y) {
            return Some(ControlPoint::TopRight);
        }

        // Bottom Left
        if is_within_point(x_left, y_bottom, x, y) {
            return Some(ControlPoint::BottomLeft);
        }

        // Bottom Right
        if is_within_point(x_right, y_bottom, x, y) {
            return Some(ControlPoint::BottomRight);
        }

        None
    }

    fn draw_dot(&self, ctx: &cairo::Context, x: f64, y: f64) {
        ctx.set_source_rgba(0.12, 0.32, 0.8, 1.0);
        ctx.arc(x, y, CONTROL_POINT_RADIUS, 0.0, 2.0 * std::f64::consts::PI);
        ctx.fill().unwrap();
    }
}

impl Drawable for CropDrawable {
    fn draw(&self, ctx: &cairo::Context, surface: &ImageSurface) {
        ctx.set_source_rgba(0.0, 0.0, 0.0, 0.5);
        ctx.set_fill_rule(cairo::FillRule::EvenOdd);

        ctx.rectangle(0.0, 0.0, surface.width() as f64, surface.height() as f64);
        ctx.rectangle(self.x1, self.y1, self.x2 - self.x1, self.y2 - self.y1);
        ctx.fill().unwrap();

        ctx.set_source_rgba(0.12, 0.32, 0.8, 1.0);
        ctx.rectangle(self.x1, self.y1, self.x2 - self.x1, self.y2 - self.y1);
        ctx.set_line_width(0.75);
        ctx.stroke().unwrap();

        // four dots
        self.draw_dot(ctx, self.x1, self.y1);
        self.draw_dot(ctx, self.x2, self.y1);
        self.draw_dot(ctx, self.x1, self.y2);
        self.draw_dot(ctx, self.x2, self.y2);
    }

    fn draw_final(&self, _ctx: &cairo::Context, _surface: &ImageSurface) {
        // We won't draw anything to the final screenshot
    }
}
