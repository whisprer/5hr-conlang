use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum RrlangError {
    Io(std::io::Error),
    Message(String),
}

pub type Result<T> = std::result::Result<T, RrlangError>;

impl From<std::io::Error> for RrlangError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl Display for RrlangError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RrlangError::Io(err) => write!(f, "I/O error: {err}"),
            RrlangError::Message(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for RrlangError {}
