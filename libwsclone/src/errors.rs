use std::fmt::Formatter;

#[derive(Debug, PartialEq)]
pub enum WscError {
    ErrorCreatingDestinationDirectory(String),
    InvalidHtml(String),
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
    ChannelClosed,
    InvalidUrl(String),
}

impl std::fmt::Display for WscError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            WscError::ErrorCreatingDestinationDirectory(err) => {
                format!("error creating destination directory. {}", err)
            }
            WscError::InvalidHtml(f_name) => {
                format!("error processing page content in file {}", f_name)
            }
            WscError::UnknownError(err) => format!("an unknown error occurred. {}", err),
            WscError::DestinationDirectoryDoesNotExist(dir) => format!(
                "the provided destination directory {}, does not exist.",
                dir
            ),
            WscError::FileOperationError { file_name, message } => {
                format!("{} : {}", message, file_name)
            }
            WscError::NetworkError(err) => format!("error connecting to internet. {}", err),
            WscError::ErrorStatusCode { status_code, url } => format!(
                "server returned an error response. {} => {}",
                url, status_code
            ),
            WscError::ChannelClosed => format!("Channel closed before download completion"),
        };
        write!(f, "{}", str)
    }
}

impl std::error::Error for WscError {}
