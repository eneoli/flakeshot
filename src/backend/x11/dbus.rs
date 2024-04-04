use anyhow::Result;
use zbus::blocking::Connection;

pub fn try_create_screenshots() -> Result<Vec<(OutputInfo, image::DynamicImage)>, Error> {
    let connection = Connection::session()?;

    todo!()
}
