trait Drawable {
    fn draw();
}


struct Canavs {
    drawables: Vec<&Box<dyn Drawable>>
}

impl Canvas {
    pub fn draw(&self) {
        for drawable in self.drawables.iter() {
            drawable.draw();
        }
    }
}


trait Tool {
    fn get_drawable(drawables: &mut HashMap<u32, Drawable>): Drawable; // TODO

    fn handle_event();
}


struct CropDrawable {}

struct Crop {
    drawable: Box<CropDrawable>,
}

impl Tool for Crop {
    fn get_drawable(&self) -> &Box<dyn Drawable> {
        &self.drawable as Box<dyn D
    }

    fn handle_event(&mut self) {
        // change self.drawable
    }
}

struct Manager {
    active_tool: Box<dyn Tool>,
}
