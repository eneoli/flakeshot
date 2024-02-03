use cairo::ImageSurface;

use crate::frontend::{rectangle::Rectangle, ui::drawable::Drawable};

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
    drawable: CropDrawable,
    is_active: bool,
    is_within: bool,
    is_dragging: bool,
    mouse_x: f64,
    mouse_y: f64,
}

impl Default for Crop {
    fn default() -> Self {
        Self::new()
    }
}

impl Crop {
    pub fn new() -> Self {
        Self {
            drawable: CropDrawable::new(),
            is_active: false,
            is_within: false,
            is_dragging: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
        }
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl Tool for Crop {
    fn handle_mouse_move(&mut self, x: f64, y: f64) -> ToolCommand {
        let Rectangle { x1, x2, y1, y2 } = &mut self.drawable.selection;

        if self.is_active && !self.is_dragging {
            *x2 = x;
            *y2 = y;
        }

        if self.is_within && self.is_dragging {
            // move box
            let delta_x = x - self.mouse_x;
            let delta_y = y - self.mouse_y;

            *x1 += delta_x;
            *x2 += delta_x;
            *y1 += delta_y;
            *y2 += delta_y;
        }

        self.is_within = self.drawable.is_within_selection(x, y);

        self.mouse_x = x;
        self.mouse_y = y;

        ToolCommand::Noop
    }

    fn handle_mouse_press(&mut self, x: f64, y: f64) -> ToolCommand {
        if !self.is_within {
            self.is_active = true;

            let control_point = self.drawable.is_within_control_point(x, y);

            if let Some(control_point) = control_point {
                let (x_left, x_right, y_top, y_bottom) =
                    self.drawable.selection.get_points_clockwise();

                // Extend current box
                let Rectangle { x1, x2, y1, y2 } = &mut self.drawable.selection;
                match control_point {
                    ControlPoint::TopLeft => {
                        *x1 = x_right;
                        *y1 = y_bottom;
                    }
                    ControlPoint::TopRight => {
                        *x1 = x_left;
                        *y1 = y_bottom;
                    }
                    ControlPoint::BottomLeft => {
                        *x1 = x_right;
                        *y1 = y_top;
                    }
                    ControlPoint::BottomRight => {
                        *x1 = x_left;
                        *y1 = y_top;
                    }
                }

                *x2 = x;
                *y2 = y;
            } else {
                // Create new box
                let Rectangle { x1, x2, y1, y2 } = &mut self.drawable.selection;
                *x1 = x;
                *y1 = y;
                *x2 = x;
                *y2 = y;
            }

            self.is_dragging = false;
        } else {
            self.is_dragging = true;
        }

        ToolCommand::Noop
    }

    fn handle_mouse_release(&mut self, x: f64, y: f64) -> ToolCommand {
        self.is_active = false;
        self.is_dragging = false;

        let Rectangle { x2, y2, .. } = &mut self.drawable.selection;

        if !self.is_within {
            *x2 = x;
            *y2 = y;
        }

        ToolCommand::Crop(self.drawable.selection.clone())
    }

    fn get_drawable(&self) -> &dyn Drawable {
        &self.drawable
    }
}

pub struct CropDrawable {
    pub selection: Rectangle,
}

impl Default for CropDrawable {
    fn default() -> Self {
        Self::new()
    }
}

impl CropDrawable {
    pub fn new() -> Self {
        CropDrawable {
            selection: Rectangle::new(),
        }
    }

    pub fn is_within_selection(&self, x: f64, y: f64) -> bool {
        let (x_left, x_right, y_top, y_bottom) = self.selection.get_points_clockwise();

        x_left <= x
            && x_right >= x
            && y_top <= y
            && y_bottom >= y
            && self.is_within_control_point(x, y).is_none()
    }

    fn is_within_control_point(&self, x: f64, y: f64) -> Option<ControlPoint> {
        let is_within_point = |xc: f64, yc: f64, x: f64, y: f64| {
            let radius = CONTROL_POINT_RADIUS + CONTROL_POINT_TOLERANCE;

            xc - radius <= x && xc + radius >= x && yc - radius <= y && yc + radius >= y
        };

        let (x_left, x_right, y_top, y_bottom) = self.selection.get_points_clockwise();

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

    fn draw(&self, acitve: bool, ctx: &cairo::Context, surface: &ImageSurface) {
        let Rectangle { x1, x2, y1, y2 } = self.selection;

        ctx.set_source_rgba(0.0, 0.0, 0.0, 0.5);
        ctx.set_fill_rule(cairo::FillRule::EvenOdd);

        ctx.rectangle(0.0, 0.0, surface.width() as f64, surface.height() as f64);
        ctx.rectangle(x1, y1, x2 - x1, y2 - y1);
        ctx.fill().unwrap();

        ctx.set_source_rgba(0.12, 0.32, 0.8, 1.0);
        ctx.rectangle(x1, y1, x2 - x1, y2 - y1);
        ctx.set_line_width(0.75);
        ctx.stroke().unwrap();

        // four dots
        if acitve {
            self.draw_dot(ctx, x1, y1);
            self.draw_dot(ctx, x2, y1);
            self.draw_dot(ctx, x1, y2);
            self.draw_dot(ctx, x2, y2);
        }
    }

    fn draw_dot(&self, ctx: &cairo::Context, x: f64, y: f64) {
        ctx.set_source_rgba(0.12, 0.32, 0.8, 1.0);
        ctx.arc(x, y, CONTROL_POINT_RADIUS, 0.0, 2.0 * std::f64::consts::PI);
        ctx.fill().unwrap();
    }
}

impl Drawable for CropDrawable {
    fn draw_active(&self, ctx: &cairo::Context, surface: &ImageSurface) {
        self.draw(true, ctx, surface);
    }

    fn draw_inactive(&self, ctx: &cairo::Context, surface: &ImageSurface) {
        self.draw(false, ctx, surface);
    }

    fn draw_final(&self, _ctx: &cairo::Context, _surface: &ImageSurface) {
        // We won't draw anything to the final screenshot
    }
}
