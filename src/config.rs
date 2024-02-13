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
    pub clipman: Clipman,
}

impl Default for X11 {
    fn default() -> Self {
        let cmd = ["xclip", "-selection", "clipboard", "-target", "image/png"];

        Self {
            clipman: Clipman(cmd.into_iter().map(|s| s.to_string()).collect()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Wayland {
    pub clipman: Clipman,
}

impl Default for Wayland {
    fn default() -> Self {
        Self {
            clipman: Clipman(vec!["wl-copy".to_string()]),
        }
    }
}

/// Stores the clipboard maager command.
///
/// # Invariant
/// It's always garanteed that the vector has at least one element (the command)!
#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "Self", transparent)]
pub struct Clipman(Vec<String>);

impl Clipman {
    pub fn cmd(&self) -> &String {
        &self.0[0]
    }

    pub fn args(&self) -> &[String] {
        &self.0[1..]
    }
}

impl<'de> Deserialize<'de> for Clipman {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let unchecked = Clipman::deserialize(deserializer)?;

        if unchecked.0.is_empty() {
            return Err(serde::de::Error::custom(
                "Clipboard-manager command can't be empty!",
            ));
        }

        Ok(unchecked)
    }
}

impl Serialize for Clipman {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Clipman::serialize(self, serializer)
    }
}

pub fn print_default_config() {
    println!("{}", toml::to_string_pretty(&Config::default()).unwrap());
}
