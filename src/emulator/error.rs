use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmulatorError {
    InvalidCommand(u16),
}

impl Display for EmulatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EmulatorError::InvalidCommand(c) => write!(f, "Invalid command: {:#06x}", c)
        }
    }
}
