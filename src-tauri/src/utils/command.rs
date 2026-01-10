use anyhow::Error;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
pub struct CommandError(String);

impl From<Error> for CommandError {
    fn from(error: Error) -> Self {
        Self(format!("{:#?}", error))
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
