use std::{
    io::Error as IoError,
    result::{
        Result as StdResult
    },
};

/// The kinds of command errors.
#[derive(Debug)]
pub enum CommandErrorKind {
    Io(IoError),
    UnimplementedCommand,
    UnimplementedAbstractionLayer,
}

/// An error that can be encountered while executing commands.
#[derive(Debug)]
pub struct CommandError {
    kind: CommandErrorKind,
}

impl From<IoError> for CommandError {
    fn from(error: IoError) -> Self {
        Self {
            kind: CommandErrorKind::Io(error),
        }
    }
}

impl CommandError {
    /// Creates a new command error from the given kind.
    pub fn new(kind: CommandErrorKind) -> Self {
        Self { kind }
    }
}

/// Result type that has a `CommandError`.
pub type Result<T> = StdResult<T, CommandError>;
