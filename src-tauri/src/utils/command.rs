use anyhow::Error;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommandError(String);

impl From<Error> for CommandError {
    fn from(error: Error) -> Self {
        Self(format!("{:#?}", error))
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
