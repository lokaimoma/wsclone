#[derive(Debug, PartialEq)]
pub enum WscError {
    ErrorDownloadingResource(String),
    ErrorCreatingDestinationDirectory(String),
    InvalidHtml,
    UnknownError(String),
    /// Parameter is path to directory
    DestinationDirectoryDoesNotExist(String),
    /// parameters are file path, additional error message
    FileOperationError(String, String),
}
