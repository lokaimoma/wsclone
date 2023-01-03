#[derive(Debug, PartialEq)]
pub enum WscError {
    ResourceAlreadyRegistered,
    FailedToConnectToServer(String),
    ErrorDownloadingResource(String),
    ErrorFetchingResourceInfo(String),
    ErrorParsingIndexUrl(String),
    ErrorCreatingDestinationDirectory(String),
    InvalidHtml,
    UnknownError(String),
    /// Parameter is path to directory
    DestinationDirectoryDoesNotExist(String),
    /// parameters are file path, additional error message
    FileOperationError(String, String),
}
