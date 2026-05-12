use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Message(String),
    Cancelled,
    InvalidSelection(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Message(msg) => write!(f, "{msg}"),
            Self::Cancelled => write!(f, "No files were restored"),
            Self::InvalidSelection(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
