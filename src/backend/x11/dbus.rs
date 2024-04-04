use std::collections::HashMap;

use anyhow::Result;
use zbus::blocking::Connection;
use zbus_macros::proxy;
use zvariant::{OwnedObjectPath, Str, Value};

use crate::backend::OutputInfo;

use super::Error;

#[proxy(
    interface = "org.freedesktop.portal.Screenshot",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Screenshot {
    fn pick_color(
        &self,
        parent_window: &str,
        options: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;

    fn screenshot(
        &self,
        parent_window: &str,
        options: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;

    #[zbus(property, name = "version")]
    fn version(&self) -> zbus::Result<u32>;
}

pub fn try_create_screenshots() -> Result<Vec<(OutputInfo, image::DynamicImage)>, Error> {
    let conn = Connection::session()?;

    let screenshot = ScreenshotProxyBlocking::new(&conn).unwrap();
    let options = HashMap::from([
        ("interactive", Value::Bool(false)),
        ("handle_token", Value::Str(Str::from("demo_app"))),
    ]);
    let path = screenshot.screenshot("", options)?;

    let dest = String::new();
    conn.call_method(
        Some("org.freedesktop.portal.Desktop"),
        path,
        Some("org.freedesktop.portal.Request"),
        "Response",
        &dest,
    )
    .unwrap();

    println!("{}", dest);

    todo!("Did a path could be printed out?")
}
