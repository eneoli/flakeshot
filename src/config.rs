use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::{error, warn};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub x11: X11,
    pub wayland: Wayland,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).map_err(|e| {
                error!("Couldn't parse config file: {}", e);
                e.into()
            }),
            Err(e) => {
                warn!(
                    "Couldn't read config file at {}: {}",
                    path.as_ref().to_string_lossy(),
                    e
                );
                Err(e.into())
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct X11 {
    pub clipboard: Clipboard,
}

impl Default for X11 {
    fn default() -> Self {
        Self {
            clipboard: Clipboard {
                cmd: "xclip".to_string(),
                args: ["-selection", "clipboard", "-target", "image/png"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Wayland {
    pub clipboard: Clipboard,
}

impl Default for Wayland {
    fn default() -> Self {
        Self {
            clipboard: Clipboard {
                cmd: "wl-copy".to_string(),
                args: vec![],
            },
        }
    }
}

/// Stores the clipboard maager command.
///
/// # Invariant
/// It's always garanteed that the vector has at least one element (the command)!
#[derive(Debug, Serialize, Deserialize)]
pub struct Clipboard {
    pub cmd: String,
    pub args: Vec<String>,
}

pub fn print_default_config() {
    println!("{}", toml::to_string_pretty(&Config::default()).unwrap());
}
