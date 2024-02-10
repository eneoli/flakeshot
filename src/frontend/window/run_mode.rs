use crate::cli::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Tray,
    Gui,
}

impl From<Command> for RunMode {
    fn from(value: Command) -> Self {
        match value {
            Command::Gui => Self::Gui,
            Command::Tray => Self::Tray,
            _ => unreachable!("There's  no run mode for {:?} defined.", value),
        }
    }
}
