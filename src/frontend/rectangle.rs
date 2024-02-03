#[derive(Clone)]
pub struct Rectangle {
    pub x1: f64,
    pub x2: f64,
    pub y1: f64,
    pub y2: f64,
}

impl Rectangle {
    pub fn new() -> Self {
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn with_size(width: f64, height: f64) -> Self {
        Self {
            x1: 0.0,
            x2: width,
            y1: 0.0,
            y2: height,
        }
    }

    pub fn get_points_clockwise(&self) -> (f64, f64, f64, f64) {
        let x_left = f64::min(self.x1, self.x2);
        let x_right = f64::max(self.x1, self.x2);
        let y_top = f64::min(self.y1, self.y2);
        let y_bottom = f64::max(self.y1, self.y2);

        (x_left, x_right, y_top, y_bottom)
    }
}
