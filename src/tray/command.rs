use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    CreateScreenshot,

    /// Send a notifiaction to the user
    Notify(String),
}
