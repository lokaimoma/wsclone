#[derive(Debug)]
pub enum WscError {
    ResourceAlreadyRegistered(),
    FailedToConnectToServer(String),
    ErrorDownloadingResource(String),
    ErrorFetchingResourceInfo(String),
    ErrorParsingIndexUrl(String),
    /// Parameter is path to directory
    DestinationDirectoryDoesNotExist(String),
    /// parameters are file path, additional error message
    FailedToOpenResourceFile(String, String),
    ErrorWritingToFile(String, String),
    ErrorReadingHtmlFile(String, String),
}
