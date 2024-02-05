use gtk4::cairo::{Context, FillRule, ImageSurface};

use crate::frontend::{
    shape::{point::Point, rectangle::Rectangle},
    ui::drawable::Drawable,
};

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
    mouse_pos: Point,
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
            mouse_pos: Point::default(),
        }
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl Tool for Crop {
    fn handle_mouse_move(&mut self, position: Point) -> ToolCommand {
        if self.is_active && !self.is_dragging {
            self.drawable.selection.snd = position;
        }

        if self.is_within && self.is_dragging {
            // move box
            let delta_x = position.x - self.mouse_pos.x;
            let delta_y = position.y - self.mouse_pos.y;

            self.drawable.selection.fst.add(Point {
                x: delta_x,
                y: delta_y,
            });

            self.drawable.selection.snd.add(Point {
                x: delta_x,
                y: delta_y,
            });
        }

        self.is_within = self.drawable.is_within_selection(position);

        self.mouse_pos = position;

        ToolCommand::Noop
    }

    fn handle_mouse_press(&mut self, position: Point) -> ToolCommand {
        if !self.is_within {
            self.is_active = true;

            let control_point = self.drawable.is_within_control_point(position);

            if let Some(control_point) = control_point {
                let (fst, snd) = self.drawable.selection.get_points_clockwise();

                // Extend current box
                self.drawable.selection.fst = match control_point {
                    ControlPoint::TopLeft => Point {
                        x: snd.x,
                        y: snd.y,
                    },
                    ControlPoint::TopRight => Point {
                        x: fst.x,
                        y: snd.y,
                    },
                    ControlPoint::BottomLeft => Point {
                        x: snd.x,
                        y: fst.y,
                    },
                    ControlPoint::BottomRight => Point {
                        x: fst.x,
                        y: fst.y,
                    },
                };

                self.drawable.selection.snd = position;
            } else {
                // Create new box
                self.drawable.selection = Rectangle {
                    fst: position,
                    snd: position,
                };
            }

            self.is_dragging = false;
        } else {
            self.is_dragging = true;
        }

        ToolCommand::Noop
    }

    fn handle_mouse_release(&mut self, position: Point) -> ToolCommand {
        self.is_active = false;
        self.is_dragging = false;

        if !self.is_within {
            self.drawable.selection.snd = position;
        }

        ToolCommand::Crop(self.drawable.selection)
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
            selection: Rectangle::default(),
        }
    }

    pub fn is_within_selection(&self, position: Point) -> bool {
        let (fst, snd) = self.selection.get_points_clockwise();
        let Point {x, y} = position;

        fst.x <= x
            && snd.x >= x
            && fst.y <= y
            && snd.y >= y
            && self.is_within_control_point(position).is_none()
    }

    fn is_within_control_point(&self, position: Point) -> Option<ControlPoint> {
        let is_within_point = |xc: f64, yc: f64, point: Point| {
            let radius = CONTROL_POINT_RADIUS + CONTROL_POINT_TOLERANCE;

            xc - radius <= point.x && xc + radius >= point.x && yc - radius <= point.y && yc + radius >= point.y
        };

        let (fst, snd) = self.selection.get_points_clockwise();

        // Top Left
        if is_within_point(fst.x, fst.y, position) {
            return Some(ControlPoint::TopLeft);
        }

        // Top Right
        if is_within_point(snd.x, fst.y, position) {
            return Some(ControlPoint::TopRight);
        }

        // Bottom Left
        if is_within_point(fst.x, snd.y, position) {
            return Some(ControlPoint::BottomLeft);
        }

        // Bottom Right
        if is_within_point(snd.x, snd.y, position) {
            return Some(ControlPoint::BottomRight);
        }

        None
    }

    fn draw(&self, acitve: bool, ctx: &Context, surface: &ImageSurface) {
        let Rectangle { fst, snd} = self.selection;

        ctx.set_source_rgba(0.0, 0.0, 0.0, 0.5);
        ctx.set_fill_rule(FillRule::EvenOdd);

        ctx.rectangle(0.0, 0.0, surface.width() as f64, surface.height() as f64);
        ctx.rectangle(fst.x, fst.y, snd.x - fst.x, snd.y - fst.y);
        ctx.fill().unwrap();

        ctx.set_source_rgba(0.12, 0.32, 0.8, 1.0);
        ctx.rectangle(fst.x, fst.y, snd.x - fst.x, snd.y - fst.y);
        ctx.set_line_width(0.75);
        ctx.stroke().unwrap();

        // four dots
        if acitve {
            self.draw_dot(ctx, fst.x, fst.y);
            self.draw_dot(ctx, snd.x, fst.y);
            self.draw_dot(ctx, fst.x, snd.y);
            self.draw_dot(ctx, snd.x, snd.y);
        }
    }

    fn draw_dot(&self, ctx: &Context, x: f64, y: f64) {
        ctx.set_source_rgba(0.12, 0.32, 0.8, 1.0);
        ctx.arc(x, y, CONTROL_POINT_RADIUS, 0.0, 2.0 * std::f64::consts::PI);
        ctx.fill().unwrap();
    }
}

impl Drawable for CropDrawable {
    fn draw_active(&self, ctx: &Context, surface: &ImageSurface) {
        self.draw(true, ctx, surface);
    }

    fn draw_inactive(&self, ctx: &Context, surface: &ImageSurface) {
        self.draw(false, ctx, surface);
    }

    fn draw_final(&self, _ctx: &Context, _surface: &ImageSurface) {
        // We won't draw anything to the final screenshot
    }
}
