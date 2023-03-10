use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    SocketError(String),
    IOError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::SocketError(e) => format!("Socket Error : {e}"),
            Self::IOError(e) => format!("IOError : {e}"),
        };
        write!(f, "{msg}")
    }
}

pub type Result<T> = std::result::Result<T, Error>;
