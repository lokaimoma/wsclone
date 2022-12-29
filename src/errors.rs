#[derive(Debug)]
pub enum WscError {
    FailedToConnectToServer(String),
    ErrorDownloadingResource(String),
    ErrorFetchingResourceInfo(String),
    /// File name, Reason
    FailedToOpenResourceFile(String, String),
    ErrorWritingToFile(String, String),
}