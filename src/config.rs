use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub x11: X11,
    pub wayland: Wayland,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content).map_err(|e| e.into()),
            Err(e) => {
                warn!("Couldn't read config file: {}", e);
                Err(e.into())
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct X11 {
    pub clip_man: String,
    pub args: Vec<String>,
}

impl Default for X11 {
    fn default() -> Self {
        Self {
            clip_man: "xclip".into(),
            args: vec![
                String::from("-selection"),
                String::from("clipboard"),
                String::from("-target"),
                String::from("image/png"),
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Wayland {
    pub clip_man: String,
    pub args: Vec<String>,
}

impl Default for Wayland {
    fn default() -> Self {
        Self {
            clip_man: "wl-copy".into(),
            args: vec![],
        }
    }
}
