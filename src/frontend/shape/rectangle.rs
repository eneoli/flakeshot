use super::point::Point;

#[derive(Copy, Clone, Default)]
pub struct Rectangle {
    pub fst: Point,
    pub snd: Point,
}

impl Rectangle {
    pub fn with_size(width: f64, height: f64) -> Self {
        Self {
            fst: Point { x: 0.0, y: 0.0 },
            snd: Point {
                x: width,
                y: height,
            },
        }
    }

    pub fn get_points_clockwise(&self) -> (Point, Point) {
        let x_left = f64::min(self.fst.x, self.snd.x);
        let x_right = f64::max(self.fst.x, self.snd.x);
        let y_top = f64::min(self.fst.y, self.snd.y);
        let y_bottom = f64::max(self.fst.y, self.snd.y);

        (Point {x: x_left, y: y_top}, Point {x: x_right, y: y_bottom})
    }
}
