use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::error;

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
                error!(
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
        let cmd = ["xclip", "-selection", "clipboard", "-target", "image/png"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        Self {
            clipboard: Clipboard(cmd),
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
            clipboard: Clipboard(vec!["wl-copy".to_string()]),
        }
    }
}

/// Stores the clipboard maager command.
///
/// # Invariant
/// It's always garanteed that the vector has at least one element (the command)!
#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "Self", transparent)]
pub struct Clipboard(Vec<String>);

impl Clipboard {
    pub fn cmd(&self) -> &str {
        &self.0[0]
    }

    pub fn args(&self) -> &[String] {
        &self.0[1..]
    }
}

impl Serialize for Clipboard {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Clipboard::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Clipboard {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let unchecked = Clipboard::deserialize(deserializer)?;

        if unchecked.0.is_empty() {
            return Err(serde::de::Error::custom(
                "There has to be at least one value (which is the command.)",
            ));
        }

        Ok(unchecked)
    }
}

pub fn print_default_config() {
    println!("{}", toml::to_string_pretty(&Config::default()).unwrap());
}
