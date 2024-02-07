use crate::cli::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Tray,
    Gui,
}

impl From<Command> for Mode {
    fn from(value: Command) -> Self {
        match value {
            Command::Gui => Self::Gui,
            Command::Tray => Self::Tray,
        }
    }
}
