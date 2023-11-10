pub mod wayland;
pub mod xorg;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Xorg(#[from] xorg::Error),
}

pub fn get_images() -> Result<Vec<image::RgbImage>, Error> {
    let is_running_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();

    if is_running_wayland {
        todo!()
    } else {
        xorg::get_images().map_err(Error::from)
    }
}
