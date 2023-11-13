use wayland_client::protocol::wl_output::WlOutput;
use crate::backend::geometry::Geometry;
use crate::backend::output_mode::OutputMode;

#[derive(Debug)]
pub struct OutputInfo {
    pub output: WlOutput,
    pub name: String,
    pub description: String,
    pub scale: i32,
    pub geometry: Geometry,
    pub mode: OutputMode,

    // width
    // height
    // name
    // desc
    // x
    // y
    // scale
}

(OutputInfo, DynamicImage)

impl OutputInfo {
    pub fn from_wl_output(wl_ouput: WlOutput) -> OutputInfo {
        Self {
            output: wl_ouput,
            scale: 0,
            description: String::new(),
            name: String::new(),
            geometry: Geometry::default(),
            mode: OutputMode::default(),
        }
    }
}