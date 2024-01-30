pub mod file_chooser;
pub mod main_window;
pub mod screenshot_window;
pub mod ui;
pub mod ui_manager;

#[derive(thiserror::Error, Debug)]
pub enum Error {}
