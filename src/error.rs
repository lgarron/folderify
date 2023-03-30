#[derive(Debug)]
pub enum FolderifyError {
    CommandInvalid(CommandInvalidError),
    CommandFailed(CommandFailedError),
    General(GeneralError),
}

#[derive(Debug)]
pub struct CommandInvalidError {
    pub command_name: String,
}

impl From<CommandInvalidError> for FolderifyError {
    fn from(value: CommandInvalidError) -> Self {
        FolderifyError::CommandInvalid(value)
    }
}

#[derive(Debug)]
pub struct CommandFailedError {
    pub command_name: String,
    pub stderr: Vec<u8>,
}

impl From<CommandFailedError> for FolderifyError {
    fn from(value: CommandFailedError) -> Self {
        FolderifyError::CommandFailed(value)
    }
}

#[derive(Debug)]
pub struct GeneralError {
    pub message: String,
}

impl From<GeneralError> for FolderifyError {
    fn from(value: GeneralError) -> Self {
        FolderifyError::General(value)
    }
}
