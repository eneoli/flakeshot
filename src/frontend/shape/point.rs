#[derive(Debug, Copy, Clone, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn add(&mut self, point: Point) {
        self.x += point.x;
        self.y += point.y;
    }
}