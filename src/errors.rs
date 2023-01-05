#[derive(Debug, PartialEq)]
pub enum WscError {
    ErrorCreatingDestinationDirectory(String),
    InvalidHtml,
    UnknownError(String),
    /// Parameter is path to directory
    DestinationDirectoryDoesNotExist(String),
    /// parameters are file path, additional error message
    FileOperationError {
        file_name: String,
        message: String,
    },
    NetworkError(String),
    ErrorStatusCode {
        status_code: String,
        url: String,
    },
}
