use std::{
    fmt,
    path::PathBuf,
};

#[derive(PartialEq)]
pub enum Error {
    EmptySource,
    NoSuchFile(PathBuf),
    CannotRead(PathBuf),
    ParseError(qasm::Error),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EmptySource =>
                write!(f, "Given an empty source"),
            Error::NoSuchFile(file) =>
                write!(f, "File \"{file:?}\" not found", file=file),
            Error::CannotRead(file) =>
                write!(f, "File \"{file:?}\" is unreadable", file=file),
            Error::ParseError(err) =>
                write!(f, "Parser error: {err:?}", err=err),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;