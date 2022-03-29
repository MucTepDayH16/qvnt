use std::{fmt, path::PathBuf};

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptySource,
    NoSuchFile(PathBuf),
    CannotRead(PathBuf),
    ParseError(qasm::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EmptySource => write!(f, "Given an empty source"),
            Error::NoSuchFile(file) => write!(f, "File {file:?} not found"),
            Error::CannotRead(file) => write!(f, "Cannot read file {file:?}"),
            Error::ParseError(err) => write!(f, "Parser error: {err:?}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
