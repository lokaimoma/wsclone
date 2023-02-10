use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    InvalidPayload(String),
    MessageIO(String),
    Serialization(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::MessageIO(reason) => {
                format!("Error reading message from stream for reason : {reason}")
            }
            Self::InvalidPayload(reason) => reason.to_string(),
            Self::Serialization(reason) => reason.to_string(),
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
