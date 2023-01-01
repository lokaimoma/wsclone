#[derive(Debug)]
pub enum WscError {
    FailedToConnectToServer(String),
    ErrorDownloadingResource(String),
    ErrorFetchingResourceInfo(String),
    ErrorParsingIndexUrl(String),
    /// File path, Reason
    FailedToOpenResourceFile(String, String),
    ErrorWritingToFile(String, String),
    ErrorReadingHtmlFile(String, String),
}
